use std::time::Duration;
use wazir_drop::clock::{Stopwatch, Timer};

#[test]
fn test_stopwatch() {
    let mut stopwatch = Stopwatch::new();
    std::thread::sleep(Duration::from_millis(200));
    stopwatch.start();
    std::thread::sleep(Duration::from_millis(200));
    let t = stopwatch.get();
    assert!(t > Duration::from_millis(150));
    assert!(t < Duration::from_millis(250));
    stopwatch.stop();
    std::thread::sleep(Duration::from_millis(200));
    let t = stopwatch.get();
    assert!(t > Duration::from_millis(150));
    assert!(t < Duration::from_millis(250));
}

#[test]
fn test_timer() {
    let mut timer = Timer::new(Duration::from_millis(300));
    std::thread::sleep(Duration::from_millis(200));
    timer.start();
    std::thread::sleep(Duration::from_millis(200));
    let t = timer.get();
    assert!(t > Duration::from_millis(50));
    assert!(t < Duration::from_millis(150));
    timer.stop();
    std::thread::sleep(Duration::from_millis(200));
    let t = timer.get();
    assert!(t > Duration::from_millis(50));
    assert!(t < Duration::from_millis(150));
    timer.start();
    std::thread::sleep(Duration::from_millis(200));
    let t = timer.get();
    assert_eq!(t, Duration::ZERO);
}
