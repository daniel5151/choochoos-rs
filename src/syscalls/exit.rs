pub fn exit(scheduler: &mut crate::Scheduler) {
    scheduler.exit_current_task();
}
