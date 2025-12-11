use mysql::*;
use mysql::prelude::*;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub chat_role: String,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: i32,
    pub title: String,
    pub is_group: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: i32,
    pub conversation_id: i32,
    pub user_id: i32,
    pub content: String,
    pub reaction: Option<String>,
    pub reply_to_id: Option<i32>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationUser {
    pub conversation_id: i32,
    pub user_id: i32,
    pub is_admin: bool,
}

pub struct Database {
    pool: Pool,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
        let db_name = env::var("DB_NAME").unwrap_or_else(|_| "tunispace".to_string());
        let db_user = env::var("DB_USER").unwrap_or_else(|_| "root".to_string());
        let db_pass = env::var("DB_PASS").unwrap_or_else(|_| "".to_string());

        let url = format!(
            "mysql://{}:{}@{}/{}",
            db_user, db_pass, db_host, db_name
        );

        let pool = Pool::new(url.as_str())?;
        Ok(Database { pool })
    }

    // ===== USER OPERATIONS =====

    pub fn find_user_by_id(&self, user_id: i32) -> Result<Option<User>> {
        let mut conn = self.pool.get_conn()?;
        let result = conn.exec_first(
            "SELECT id, username, email, COALESCE(password, '') as password, chat_role, is_active,
                    DATE_FORMAT(created_at, '%Y-%m-%d %H:%i:%s') as created_at
             FROM user WHERE id = :id",
            params! { "id" => user_id },
        )?;

        Ok(result.map(|(id, username, email, password, chat_role, is_active, created_at)| User {
            id,
            username,
            email,
            password,
            chat_role,
            is_active,
            created_at,
        }))
    }

    pub fn find_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let mut conn = self.pool.get_conn()?;
        let result = conn.exec_first(
            "SELECT id, username, email, COALESCE(password, '') as password, chat_role, is_active,
                    DATE_FORMAT(created_at, '%Y-%m-%d %H:%i:%s') as created_at
             FROM user WHERE email = :email",
            params! { "email" => email },
        )?;

        Ok(result.map(|(id, username, email, password, chat_role, is_active, created_at)| User {
            id,
            username,
            email,
            password,
            chat_role,
            is_active,
            created_at,
        }))
    }

    pub fn find_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let mut conn = self.pool.get_conn()?;
        let result = conn.exec_first(
            "SELECT id, username, email, COALESCE(password, '') as password, chat_role, is_active,
                    DATE_FORMAT(created_at, '%Y-%m-%d %H:%i:%s') as created_at
             FROM user WHERE username = :username",
            params! { "username" => username },
        )?;

        Ok(result.map(|(id, username, email, password, chat_role, is_active, created_at)| User {
            id,
            username,
            email,
            password,
            chat_role,
            is_active,
            created_at,
        }))
    }

    pub fn search_users(&self, term: &str, exclude_user_id: Option<i32>) -> Result<Vec<User>> {
        let mut conn = self.pool.get_conn()?;
        let search_term = format!("%{}%", term);

        let query = if let Some(exclude_id) = exclude_user_id {
            conn.exec_map(
                "SELECT id, username, email, COALESCE(password, '') as password, chat_role, is_active,
                        DATE_FORMAT(created_at, '%Y-%m-%d %H:%i:%s') as created_at
                 FROM user
                 WHERE (username LIKE :term OR email LIKE :term) AND id <> :exclude
                 ORDER BY username ASC LIMIT 50",
                params! { "term" => &search_term, "exclude" => exclude_id },
                |(id, username, email, password, chat_role, is_active, created_at)| User {
                    id, username, email, password, chat_role, is_active, created_at,
                },
            )?
        } else {
            conn.exec_map(
                "SELECT id, username, email, COALESCE(password, '') as password, chat_role, is_active,
                        DATE_FORMAT(created_at, '%Y-%m-%d %H:%i:%s') as created_at
                 FROM user
                 WHERE username LIKE :term OR email LIKE :term
                 ORDER BY username ASC LIMIT 50",
                params! { "term" => &search_term },
                |(id, username, email, password, chat_role, is_active, created_at)| User {
                    id, username, email, password, chat_role, is_active, created_at,
                },
            )?
        };

        Ok(query)
    }

    // ===== CONVERSATION OPERATIONS =====

    pub fn find_conversation_by_id(&self, conversation_id: i32) -> Result<Option<Conversation>> {
        let mut conn = self.pool.get_conn()?;
        let result = conn.exec_first(
            "SELECT id, title, is_group, DATE_FORMAT(created_at, '%Y-%m-%d %H:%i:%s') as created_at
             FROM conversations WHERE id = :id",
            params! { "id" => conversation_id },
        )?;

        Ok(result.map(|(id, title, is_group, created_at)| Conversation {
            id,
            title,
            is_group,
            created_at,
        }))
    }

    pub fn find_conversations_by_user(&self, user_id: i32) -> Result<Vec<Conversation>> {
        let mut conn = self.pool.get_conn()?;
        let conversations = conn.exec_map(
            "SELECT c.id, c.title, c.is_group, DATE_FORMAT(c.created_at, '%Y-%m-%d %H:%i:%s') as created_at
             FROM conversations c
             JOIN conversation_users cu ON cu.conversation_id = c.id
             WHERE cu.user_id = :uid
             ORDER BY c.created_at DESC",
            params! { "uid" => user_id },
            |(id, title, is_group, created_at)| Conversation {
                id, title, is_group, created_at,
            },
        )?;

        Ok(conversations)
    }

    pub fn user_in_conversation(&self, conversation_id: i32, user_id: i32) -> Result<bool> {
        let mut conn = self.pool.get_conn()?;
        let result: Option<i32> = conn.exec_first(
            "SELECT 1 FROM conversation_users WHERE conversation_id = :cid AND user_id = :uid",
            params! { "cid" => conversation_id, "uid" => user_id },
        )?;

        Ok(result.is_some())
    }

    pub fn get_conversation_participants(&self, conversation_id: i32) -> Result<Vec<User>> {
        let mut conn = self.pool.get_conn()?;
        let participants = conn.exec_map(
            "SELECT u.id, u.username, u.email, COALESCE(u.password, '') as password, u.chat_role, u.is_active,
                    DATE_FORMAT(u.created_at, '%Y-%m-%d %H:%i:%s') as created_at
             FROM conversation_users cu
             JOIN user u ON u.id = cu.user_id
             WHERE cu.conversation_id = :cid
             ORDER BY u.username ASC",
            params! { "cid" => conversation_id },
            |(id, username, email, password, chat_role, is_active, created_at)| User {
                id, username, email, password, chat_role, is_active, created_at,
            },
        )?;

        Ok(participants)
    }

    // ===== MESSAGE OPERATIONS =====

    pub fn find_messages_by_conversation(&self, conversation_id: i32, limit: i32) -> Result<Vec<Message>> {
        let mut conn = self.pool.get_conn()?;
        let messages = conn.exec_map(
            "SELECT id, conversation_id, user_id, content, reaction, reply_to_id,
                    DATE_FORMAT(created_at, '%Y-%m-%d %H:%i:%s') as created_at
             FROM messages
             WHERE conversation_id = :cid
             ORDER BY created_at ASC
             LIMIT :limit",
            params! { "cid" => conversation_id, "limit" => limit },
            |(id, conversation_id, user_id, content, reaction, reply_to_id, created_at)| Message {
                id, conversation_id, user_id, content, reaction, reply_to_id, created_at,
            },
        )?;

        Ok(messages)
    }

    pub fn find_message_by_id(&self, message_id: i32) -> Result<Option<Message>> {
        let mut conn = self.pool.get_conn()?;
        let result = conn.exec_first(
            "SELECT id, conversation_id, user_id, content, reaction, reply_to_id,
                    DATE_FORMAT(created_at, '%Y-%m-%d %H:%i:%s') as created_at
             FROM messages WHERE id = :id",
            params! { "id" => message_id },
        )?;

        Ok(result.map(|(id, conversation_id, user_id, content, reaction, reply_to_id, created_at)| Message {
            id, conversation_id, user_id, content, reaction, reply_to_id, created_at,
        }))
    }

    pub fn insert_message(
        &self,
        conversation_id: i32,
        user_id: i32,
        content: &str,
        reply_to_id: Option<i32>,
    ) -> Result<i32> {
        let mut conn = self.pool.get_conn()?;
        conn.exec_drop(
            "INSERT INTO messages (conversation_id, user_id, content, reply_to_id)
             VALUES (:cid, :uid, :content, :reply_to_id)",
            params! {
                "cid" => conversation_id,
                "uid" => user_id,
                "content" => content,
                "reply_to_id" => reply_to_id,
            },
        )?;

        Ok(conn.last_insert_id() as i32)
    }

    pub fn get_all_conversations(&self) -> Result<Vec<Conversation>> {
        let mut conn = self.pool.get_conn()?;
        let conversations = conn.query_map(
            "SELECT id, title, is_group, DATE_FORMAT(created_at, '%Y-%m-%d %H:%i:%s') as created_at
             FROM conversations ORDER BY created_at DESC",
            |(id, title, is_group, created_at)| Conversation {
                id, title, is_group, created_at,
            },
        )?;

        Ok(conversations)
    }

    // ===== AI HELPER METHODS =====

    pub fn get_conversation_summary(&self, conversation_id: i32, message_limit: i32) -> Result<String> {
        let messages = self.find_messages_by_conversation(conversation_id, message_limit)?;

        if messages.is_empty() {
            return Ok("No messages in this conversation.".to_string());
        }

        let mut summary = String::new();
        summary.push_str(&format!("Conversation summary (last {} messages):\n\n", messages.len()));

        for msg in messages.iter() {
            let user = self.find_user_by_id(msg.user_id)?;
            let username = user.map(|u| u.username).unwrap_or_else(|| "Unknown".to_string());
            summary.push_str(&format!("[{}] {}: {}\n", msg.created_at, username, msg.content));
        }

        Ok(summary)
    }

    pub fn search_messages(&self, conversation_id: i32, search_term: &str) -> Result<Vec<Message>> {
        let mut conn = self.pool.get_conn()?;
        let search_pattern = format!("%{}%", search_term);

        let messages = conn.exec_map(
            "SELECT id, conversation_id, user_id, content, reaction, reply_to_id,
                    DATE_FORMAT(created_at, '%Y-%m-%d %H:%i:%s') as created_at
             FROM messages
             WHERE conversation_id = :cid AND content LIKE :term
             ORDER BY created_at DESC
             LIMIT 50",
            params! { "cid" => conversation_id, "term" => &search_pattern },
            |(id, conversation_id, user_id, content, reaction, reply_to_id, created_at)| Message {
                id, conversation_id, user_id, content, reaction, reply_to_id, created_at,
            },
        )?;

        Ok(messages)
    }

    pub fn get_conversation_statistics(&self, conversation_id: i32) -> Result<serde_json::Value> {
        let mut conn = self.pool.get_conn()?;

        let total_messages: i32 = conn.exec_first(
            "SELECT COUNT(*) FROM messages WHERE conversation_id = :cid",
            params! { "cid" => conversation_id },
        )?.unwrap_or(0);

        let participant_count: i32 = conn.exec_first(
            "SELECT COUNT(*) FROM conversation_users WHERE conversation_id = :cid",
            params! { "cid" => conversation_id },
        )?.unwrap_or(0);

        let conversation = self.find_conversation_by_id(conversation_id)?
            .ok_or_else(|| {
                Error::from(mysql::error::Error::from(
                    std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Conversation not found"
                    )
                ))
            })?;

        Ok(serde_json::json!({
            "conversation_id": conversation_id,
            "title": conversation.title,
            "is_group": conversation.is_group,
            "total_messages": total_messages,
            "participant_count": participant_count,
            "created_at": conversation.created_at,
        }))
    }
}