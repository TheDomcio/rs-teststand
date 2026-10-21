//! Live-engine acceptance test for the synchronization domain.

#![cfg(feature = "live-engine")]

use std::ops::Deref;
use std::time::Duration;

use rs_teststand::{
    Engine, EnqueueResult, Error, FullQueueOption, PropValType, SyncObject, SyncObjectType,
    WaitResult,
};

/// Shuts the engine down on every exit path, including a failed assertion.
struct EngineGuard(Engine);

impl Deref for EngineGuard {
    type Target = Engine;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for EngineGuard {
    fn drop(&mut self) {
        let _ = self.0.shutdown(Duration::from_secs(2));
    }
}

#[test]
#[ignore = "requires a live engine with the synchronization server registered"]
#[allow(
    clippy::too_many_lines,
    reason = "one engine covers every synchronization primitive"
)]
fn the_primitive_objects_can_be_created_and_used() -> Result<(), Error> {
    let engine = EngineGuard(Engine::new()?);
    let manager = engine.get_sync_manager(&format!("rs-parity-{}", std::process::id()))?;
    assert!(manager.is_alive()?);
    assert_ne!(manager.machine_id()?, "");
    assert!(manager.process_id()? > 0);
    manager.sleep(0.01)?;

    let (semaphore, existed) = manager.create_semaphore("rs-parity-semaphore", 1, 1)?;
    assert!(!existed);
    assert_eq!(semaphore.name()?, "rs-parity-semaphore");
    assert_eq!(
        manager.get_semaphore("rs-parity-semaphore")?.name()?,
        "rs-parity-semaphore"
    );
    assert_eq!(
        semaphore.acquire_semaphore(0.1, None, false)?,
        WaitResult::Success
    );
    semaphore.release_semaphore()?;
    let (waiting, current, initial, maximum) = semaphore.get_info()?;
    assert!(waiting >= 0 && current >= 0 && initial == 1 && maximum == 1);

    let (mutex, existed) = manager.create_mutex("rs-parity-mutex")?;
    assert!(!existed);
    assert_eq!(mutex.name()?, "rs-parity-mutex");
    assert_eq!(
        manager.get_mutex("rs-parity-mutex")?.name()?,
        "rs-parity-mutex"
    );
    let (releaser, wait, message) = mutex.lock_mutex(0.1, None)?;
    assert_eq!(wait, WaitResult::Success);
    assert_eq!(message, "");
    drop(releaser);
    assert!(mutex.get_info()? >= 0);

    let (notification, existed) = manager.create_notification("rs-parity-notification")?;
    assert!(!existed);
    assert_eq!(notification.name()?, "rs-parity-notification");
    assert_eq!(
        manager.get_notification("rs-parity-notification")?.name()?,
        "rs-parity-notification"
    );
    notification.set(None, false, false)?;
    assert_eq!(
        notification.wait(None, 0.1, None, false)?,
        WaitResult::Success
    );
    notification.clear()?;

    let (rendezvous, existed) = manager.create_rendezvous("rs-parity-rendezvous", 1)?;
    assert!(!existed);
    assert_eq!(rendezvous.name()?, "rs-parity-rendezvous");
    assert_eq!(
        manager.get_rendezvous("rs-parity-rendezvous")?.name()?,
        "rs-parity-rendezvous"
    );
    assert_eq!(
        rendezvous.rendezvous(0.1, None, false)?,
        WaitResult::Success
    );

    let (queue, existed) = manager.create_queue("rs-parity-queue", 4)?;
    assert!(!existed);
    assert_eq!(queue.name()?, "rs-parity-queue");
    assert_eq!(
        manager.get_queue("rs-parity-queue")?.name()?,
        "rs-parity-queue"
    );

    let (batch, existed) = manager.create_batch("rs-parity-batch")?;
    assert!(!existed);
    assert_eq!(batch.name()?, "rs-parity-batch");
    assert_eq!(
        manager.get_batch("rs-parity-batch")?.name()?,
        "rs-parity-batch"
    );
    let _ = batch.default_batch_synchronization()?;
    let (waiting, in_batch) = batch.get_info()?;
    assert!(waiting >= 0 && in_batch >= 0);
    let _ = batch.get_batch_threads()?;

    let element = engine.new_property_object(PropValType::Number, false, "", 0)?;
    element.set_val_number("", 0, 42.0)?;
    let destination = engine.new_property_object(PropValType::Number, false, "", 0)?;
    let (wait, enqueue) = queue.enqueue(
        false,
        FullQueueOption::Wait,
        &element,
        false,
        0.1,
        None,
        false,
    )?;
    assert_eq!(wait, WaitResult::Success);
    assert_eq!(enqueue, EnqueueResult::Enqueued);
    assert_eq!(
        queue.dequeue(true, false, &destination, 0.1, None, false)?,
        WaitResult::Success
    );
    let (waiting_to_enqueue, waiting_to_dequeue, size_limit, count) =
        queue.get_info(None, &destination)?;
    assert!(waiting_to_enqueue >= 0 && waiting_to_dequeue >= 0);
    assert_eq!(size_limit, 4);
    assert_eq!(count, 0);

    let (second_mutex, existed) = manager.create_mutex("rs-parity-mutex-2")?;
    assert!(!existed);
    let (releasers, wait, message) = mutex.lock_mutex_group(
        &[&mutex, &second_mutex],
        "",
        "",
        Some(&manager),
        0.1,
        None,
        false,
    )?;
    assert_eq!(wait, WaitResult::Success);
    assert_eq!(message, "");
    assert_eq!(releasers.len(), 2);
    drop(releasers);

    let (second_notification, existed) = manager.create_notification("rs-parity-notification-2")?;
    assert!(!existed);
    notification.set(None, false, false)?;
    let (wait, selected, offset) = notification.wait_multiple(
        &[&notification, &second_notification],
        None,
        0.1,
        None,
        false,
    )?;
    assert_eq!(wait, WaitResult::Success);
    assert!(selected.is_some());
    assert!(offset >= 0);
    notification.clear()?;

    let (second_queue, existed) = manager.create_queue("rs-parity-queue-2", 4)?;
    assert!(!existed);
    let (wait, _) = queue.enqueue(
        false,
        FullQueueOption::Wait,
        &element,
        false,
        0.1,
        None,
        false,
    )?;
    assert_eq!(wait, WaitResult::Success);
    let (wait, selected, offset) = queue.dequeue_multiple(
        &[&queue, &second_queue],
        true,
        false,
        &destination,
        0.1,
        None,
        false,
    )?;
    assert_eq!(wait, WaitResult::Success);
    assert!(selected.is_some());
    assert!(offset >= 0);

    let found = manager.get_sync_object("rs-parity-mutex", SyncObjectType::Mutex)?;
    let SyncObject::Mutex(found) = found else {
        return Err(Error::UnexpectedType {
            expected: "a mutex",
            actual: "another synchronization object",
        });
    };
    assert_eq!(found.name()?, "rs-parity-mutex");
    Ok(())
}
