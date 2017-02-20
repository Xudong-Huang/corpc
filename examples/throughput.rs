#[macro_use]
extern crate corpc;

use std::sync::Arc;
use std::time::Instant;
use corpc::conetty::coroutine;

rpc! {
    net: Multiplex;
    rpc ack();
}

struct Server;

impl RpcSpec for Server {
    fn ack(&self) {}
}

fn main() {
    coroutine::scheduler_config().set_workers(2).set_io_workers(1);
    let addr = ("127.0.0.1", 4000);
    let server = RpcServer(Server).start(&addr).unwrap();
    let clients: Vec<_> = (0..4).map(|_| RpcClient::connect(addr).unwrap()).collect();
    let clients = Arc::new(clients);
    let mut vec = vec![];
    let now = Instant::now();
    for _i in 0..100 {
        let clients = clients.clone();
        let h = coroutine::spawn(move || {
            for j in 0..10000 {
                let idx = j & 0x03;
                match clients[idx].ack() {
                    Err(err) => println!("recv err = {:?}", err),
                    _ => {}
                }
            }
            // println!("thread done, id={}", i);
        });
        vec.push(h);
    }

    for h in vec {
        h.join().unwrap();
    }

    let dur = now.elapsed();
    let dur = dur.as_secs() as f32 + dur.subsec_nanos() as f32 / 1000_000_000.0;
    println!("{} rpc/second", 1000_000.0 / dur);

    unsafe { server.coroutine().cancel() };
    server.join().ok();
}