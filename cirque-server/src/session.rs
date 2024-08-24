use tokio::io::{AsyncReadExt, AsyncWriteExt};

use cirque_core::ServerState;
use cirque_parser::{LendingIterator, StreamParser};

use crate::transport::AnyStream;

pub(crate) struct Session {
    stream: AnyStream,
}

impl Session {
    pub(crate) fn init(stream: AnyStream) -> Self {
        Self { stream }
    }

    pub(crate) async fn run(mut self, server_state: ServerState) {
        let mut stream_parser = StreamParser::default();

        let (mut state, mut rx) = server_state.new_registering_user();

        while state.is_alive() {
            tokio::select! {
                result = self.stream.read_buf(&mut stream_parser) => {
                    let Ok(received) = result else {
                        break;
                    };

                    if received == 0 {
                        break;
                    }

                    let mut iter = stream_parser.consume_iter();
                    while let Some(message) = iter.next() {
                        let message = match message {
                            Ok(m) => m,
                            Err(err) => {
                                log::warn!("error when parsing message: {err}");
                                continue;
                            }
                        };

                        state = state.handle_message(&server_state, message);
                    }
                },
                Some(msg) = rx.recv() => {
                    if self.stream.write_all(&msg).await.is_err() {
                        break;
                    }
                }
            }
        }

        server_state.dispose_state(state);

        // handle the disconnection gracefully by sending remaining
        // messages (in case the client asked a QUIT for example)
        let mut buf = std::io::Cursor::new(Vec::<u8>::new());
        while let Ok(msg) = rx.try_recv() {
            let _ = std::io::Write::write_all(&mut buf, &msg);
        }
        // TODO: maybe tolerate a timeout to send the last messages and then force quit
        let _ = self.stream.write_all(&buf.into_inner()).await;
    }
}
