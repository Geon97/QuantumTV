use rusqlite::Connection;
use std::sync::Mutex;

pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    // 现有的方法创建示例
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: Mutex::new(conn),
        }
    }
    // 访问数据库
    pub fn with_conn<T, F>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&Connection) -> Result<T, rusqlite::Error>,
    {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("数据库锁失败: {}", e))?;
        f(&conn).map_err(|e| e.to_string())
    }
}
