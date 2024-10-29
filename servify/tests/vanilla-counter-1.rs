use tokio::{sync::{mpsc, oneshot}, task::JoinSet};

pub struct Processor {
    pub count: u32,
}

enum Message {
    IncrementAndGet { amount: u32, reply: oneshot::Sender<u32> },
    Get { reply: oneshot::Sender<u32> },
}

fn spawn(mut rx: mpsc::Receiver<Message>) {
    let mut processor = Processor { count: 0 };
    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            match message {
                Message::IncrementAndGet { amount, reply } => {
                    processor.count += amount;
                    reply.send(processor.count).unwrap();
                }
                Message::Get { reply } => {
                    reply.send(processor.count).unwrap();
                }
            }
        }
    });
}

#[tokio::test]
async fn main() {
    let (tx, rx) = mpsc::channel(32);
    spawn(rx);
    let mut set = JoinSet::new();

    for _ in 0..10 {
        let tx = tx.clone();
        set.spawn(async move {
            for _ in 0..1000 {
                let (rep_tx, rep_rx) = oneshot::channel();
                tx.send(Message::IncrementAndGet { amount: 1, reply: rep_tx }).await.unwrap();
                rep_rx.await.unwrap();
            }
        });
    }
    set.join_all().await;

    let (get_tx, get_rx) = oneshot::channel();
    tx.send(Message::Get { reply: get_tx }).await.unwrap();
    assert_eq!(get_rx.await, Ok(10000));
}
