pub mod task_last_message_id;
pub mod task_process_embeds;
pub mod task_web_push;

pub async fn start_queues() {
    async_std::task::spawn(task_last_message_id::run());
    async_std::task::spawn(task_process_embeds::run());
    async_std::task::spawn(task_web_push::run());
}
