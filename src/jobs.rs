use failure::Fail;
use uuid::Uuid;

pub enum Job {
    TestJob(TestJob)
}

pub struct TestJob {
    id: Uuid,
}

impl TestJob {
    pub fn do_work() -> Result<(), failure::Error>{
        todo!()
    }
}