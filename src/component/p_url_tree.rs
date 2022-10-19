use crate::app::{ADD_ID_KEY, ID_COUNT_KEY};
enum NodeType {
    //项目节点
    PROJECT,
    //路径
    URL,
    //样例
    CASE
}

#[derive(Default)]
pub struct ProjectUrlTree {
    pub projects:Vec<ProjectNode>,
    pub id:u64
}

pub struct ProjectNode {
    pub name:String,
    pub urls:Vec<UrlNode>,
    pub id:u64
}

pub struct UrlNode {
    pub url:String,
    pub case:Vec<CaseNode>,
    pub id:u64
}

pub struct CaseNode {
    pub title:String,
    pub id:u32
}

impl Default for ProjectNode {
    fn default() -> Self {
        let id_count: &mut u64 = data.get_persisted_mut_or_default(Id::new(ID_COUNT_KEY));
        *id_count = *id_count + 1;
        let id = *id_count;
        Self { name: "项目名称".to_string(), urls:Default::default(), id }
    }
}

impl Default for UrlNode {
    fn default() -> Self {
        let id_count: &mut u64 = data.get_persisted_mut_or_default(Id::new(ID_COUNT_KEY));
        *id_count = *id_count + 1;
        let id = *id_count;
        Self { url: "/someurl".to_string(), case: Default::default(), id }
    }
}

impl Default for CaseNode {

    fn default() -> Self {
        let id_count: &mut u64 = data.get_persisted_mut_or_default(Id::new(ID_COUNT_KEY));
        *id_count = *id_count + 1;
        Self { title: "默认方案".to_string(), id}
    }
}

