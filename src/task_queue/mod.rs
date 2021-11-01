pub mod task_last_message_id;

pub async fn start_queues() {
    async_std::task::spawn(task_last_message_id::run());
}
