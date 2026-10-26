use embassy_executor::Spawner;
use embassy_time::Timer;
use picoserve::{AppBuilder, AppRouter, routing::get};

use crate::{mk_static, storage::Auth};

static CONFIG: picoserve::Config = picoserve::Config::const_default().keep_connection_alive();

const TASKS_POOL: usize = 4;

#[derive(Clone, Copy)]
pub struct App {}

impl AppBuilder for App {
    type PathRouter = impl picoserve::routing::PathRouter;

    fn build_app(self) -> picoserve::Router<Self::PathRouter> {
        picoserve::Router::new().route(
            "/",
            get(|| async move { "<!DOCTYPE HTML><html><body><h1>Hello, world!</h1></body></html" }),
        )
    }
}

pub struct Http {
    app: App,
    stack: embassy_net::Stack<'static>,
}

impl Http {
    pub fn new(stack: embassy_net::Stack<'static>) -> Self {
        Self { app: App {}, stack }
    }

    pub async fn start<'a>(&'a self, spawner: &Spawner) -> Auth<'a> {
        let app = mk_static!(AppRouter<App>, self.app.build_app());
        for task in 0..TASKS_POOL {
            spawner.spawn(web_task(task, self.stack, app).unwrap());
        }

        Timer::after_secs(30).await;

        // mspc channel which waits for Auth here

        Auth { ssid: "Cool-network", password: "123123123" }
    }
}

#[embassy_executor::task(pool_size = TASKS_POOL)]
async fn web_task(
    task_id: usize,
    stack: embassy_net::Stack<'static>,
    app: &'static AppRouter<App>,
) -> ! {
    let port = 80;
    let mut tcp_rx_buffer = [0; 1024];
    let mut tcp_tx_buffer = [0; 1024];
    let mut http_buffer = [0; 2048];

    picoserve::Server::new(app, &CONFIG, &mut http_buffer)
        .listen_and_serve(task_id, stack, port, &mut tcp_rx_buffer, &mut tcp_tx_buffer)
        .await
        .into_never()
}
