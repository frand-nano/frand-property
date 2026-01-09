use frand_property::{Property, PropertyStreamExt, StreamExt};
use tokio::time::Duration;

#[tokio::test]
async fn test_stream_bind() {
    let source = Property::new((), 0, |_, _| {});
    let target = Property::new((), 0, |_, _| {});

    source.receiver()
        .stream()
        .map(|v| v * 2)
        .spawn_bind(target.sender().clone());

    tokio::time::sleep(Duration::from_millis(10)).await;
    assert_eq!(target.receiver().value(), 0); // 0 * 2 = 0

    source.sender().send(10);
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert_eq!(target.receiver().value(), 20);

    source.sender().send(25);
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert_eq!(target.receiver().value(), 50);
}

#[tokio::test]
async fn test_spawn_drive() {
    let source = Property::new((), 0, |_, _| {});
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    source.receiver()
        .stream()
        .map(|v| v + 1)
        .spawn(move |v| {
            let tx = tx.clone();
            async move {
                tx.send(v).unwrap();
            }
        });

    tokio::time::sleep(Duration::from_millis(10)).await;
    assert_eq!(rx.recv().await, Some(1));

    source.sender().send(10);
    assert_eq!(rx.recv().await, Some(11));
}

