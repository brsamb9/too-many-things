use crate::error_hander::Result;
use log::info;
use rand::{self, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::hash::Hash;
use std::io::BufReader;
use std::{collections::HashMap, path::PathBuf};

pub type TopicName = String;

pub trait TasksIOWrapper<'a, T: Default + Serialize + Deserialize<'a>> {
    fn storage() -> PathBuf;
    fn create_new_empty_file(file_path: PathBuf) -> Result<()>;
    fn write_to_disk(self) -> Result<()>;
    fn read_from_disk(file_path: PathBuf) -> Result<T>;
}

#[derive(Debug)]
pub struct TaskContainer {
    pub user_tasks: UserTasks,
    file_path: PathBuf,
}

impl TaskContainer {
    pub fn new() -> Result<Self> {
        let file_path = Self::storage();

        Ok(Self {
            user_tasks: Self::read_from_disk(file_path.clone())?,
            file_path,
        })
    }

    pub fn handle_create(
        &mut self,
        topic: String,
        task: String,
        task_description: Option<String>,
        link: Option<String>,
    ) -> Result<()> {
        info!("Creating {task} for {topic}");
        let topic_map = self.user_tasks.topic_hashmap.get_mut(topic.as_str());
        let task = Task::new(task, task_description, link);
        // Check branches for collison? Or leave as no-op
        match topic_map {
            Some(topic) => {
                topic.tasks.replace(task);
            }
            None => {
                self.user_tasks
                    .topic_hashmap
                    .insert(topic, Topic::from(task));
            }
        }
        Ok(())
    }

    pub fn handle_read(&self, topic: Option<String>, task: Option<String>) -> Result<()> {
        let json_print = match (topic, task) {
            (Some(top), None) => {
                info!("Reading specific topic: {top}");
                let v = self
                    .user_tasks
                    .topic_hashmap
                    .get(top.as_str())
                    .ok_or("Topic not found.")?;
                serde_json::to_string_pretty(v)
            }
            _ => {
                info!("Reading entire state");
                let v = &self.user_tasks;
                serde_json::to_string_pretty(v)
            }
        }?;
        println!("{json_print}");
        Ok(())
    }
    pub fn handle_delete(&mut self, topic: String, task: Option<String>) -> Result<()> {
        match task {
            None => {
                info!("Deleting topic: {topic}");
                self.user_tasks.topic_hashmap.remove(&topic);
            }
            Some(task) => {
                info!("Deleting task:{task:?} for topic:{topic}");
                self.user_tasks
                    .topic_hashmap
                    .get_mut(&topic)
                    .map(|f| f.tasks.remove(&Task::new(task, None, None)));
            }
        }
        Ok(())
    }
    fn random_from_list<T: Copy>(list: &Vec<T>) -> T {
        let idx = rand::thread_rng().gen_range(0..list.len());
        list[idx]
    }
    pub fn handle_randomise(&self, topic: Option<String>) -> Result<()> {
        if self.user_tasks.topic_hashmap.is_empty() {
            info!("Trying to randomise on empty.");
            // TODO: should be an error
            return Ok(());
        }

        let topic_keys: Vec<&String> = self.user_tasks.topic_hashmap.keys().collect();

        let random_topic = match &topic {
            Some(topic) if self.user_tasks.topic_hashmap.contains_key(topic) => topic,
            _ => Self::random_from_list(&topic_keys),
        };

        let task_list: Vec<&Task> = self
            .user_tasks
            .topic_hashmap
            .get(random_topic)
            .unwrap()
            .tasks
            .iter()
            .collect();

        if task_list.is_empty() {
            info!("Empty task list.");
            // TODO: should be an error
            return Ok(());
        }

        let random_task = Self::random_from_list(&task_list);
        println!("Go and do - {random_topic}:\n{random_task:#?}");

        Ok(())
    }
}

impl TasksIOWrapper<'static, UserTasks> for TaskContainer {
    // Actually just a JSON file, but maybe a DB in the future
    fn storage() -> PathBuf {
        PathBuf::from("./.tmp-storage.json")
    }

    fn create_new_empty_file(file_path: PathBuf) -> Result<()> {
        let file = File::options()
            .read(true)
            .write(true)
            .create_new(true)
            .open(file_path.as_path())?;

        let val = serde_json::to_value(UserTasks::default())?;
        serde_json::to_writer(&file, &val)?;

        Ok(())
    }

    fn write_to_disk(self) -> Result<()> {
        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(self.file_path.as_path())?;

        let val = serde_json::to_value(self.user_tasks)?;
        serde_json::to_writer(&file, &val)?;

        info!("Written: {val}");
        Ok(())
    }

    // Making this generic seemed rather difficult. Need to be more familiar with serde.
    fn read_from_disk(file_path: PathBuf) -> Result<UserTasks> {
        let file = match File::open(file_path.as_path()) {
            Ok(f) => f,
            Err(e) => {
                info!("File doesn't exist, creating empty file - {e}");
                Self::create_new_empty_file(file_path.clone())?;
                File::open(file_path.as_path()).expect("File should have existed")
            }
        };
        let reader = BufReader::new(file);
        info!("File opened, attempting to read as json");
        let v_de = serde_json::from_reader(reader)?;

        info!("read: {v_de:?}");

        Ok(v_de)
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UserTasks {
    pub topic_hashmap: HashMap<TopicName, Topic>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Topic {
    pub tasks: HashSet<Task>,
}

impl From<Task> for Topic {
    fn from(task: Task) -> Self {
        Self {
            tasks: HashSet::from([task]),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub task_name: String,
    pub task_description: Option<String>,
    pub link: Option<String>,
}
impl Task {
    fn new(task_name: String, task_description: Option<String>, link: Option<String>) -> Self {
        Self {
            task_name,
            task_description,
            link,
        }
    }
}

// Manually implemented to have a way to delete task without having to know all the other metadata apart from its name
impl Hash for Task {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.task_name.hash(state);
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.task_name == other.task_name
    }
}

impl Eq for Task {
    fn assert_receiver_is_total_eq(&self) {}
}
