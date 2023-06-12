use worker::*;

use futures::{stream::StreamExt};

mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], headers: {:?}",
        Date::now().to_string(),
        req.path(),
        req.headers(),
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    let ws_res = WebSocket::connect("wss://socketsbay.com/wss/v2/1/demo/".parse()?).await;

    let ws = match ws_res {
        Ok(ws) => ws,
        Err(e) => return Response::error(format!("failed to connect to websocket, {:?}", e), 500),
    };

    // It's important that we call this before we send our first message, otherwise we will
    // not have any event listeners on the socket to receive the echoed message.
    let mut event_stream = ws.events()?;

    ws.accept()?;
    ws.send_with_str("Hello, world!")?;

    while let Some(event) = event_stream.next().await {
        let event = event?;

        if let WebsocketEvent::Message(msg) = event {
            if let Some(text) = msg.text() {
                return Response::ok(text);
            }
        }
    }

    Response::error("never got a message echoed back :(", 500)
}
