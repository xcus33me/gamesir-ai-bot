use genai::chat::printer::{print_chat_stream, PrintChatStreamOptions};
use genai::chat::{ChatMessage, ChatRequest};
use genai::{Client, Error};
use serenity::client;

const MODEL_OPENAI: &str = "gpt-3.5-turbo";
const MODEL_GEMINI: &str = "gemini-1.5-flash";

pub enum Model {
    OpenAI,
    Gemini,
} 

impl Model {
    pub fn as_str(&self) -> &'static str {
        match self {
            Model::OpenAI => MODEL_OPENAI,
            Model::Gemini => MODEL_GEMINI,
        }
    }
}

pub struct AIClient {
    model: Model,
    client: Client,
}

const SYSTEM_PROMPT: &str = "Ты — дерзкий и задиристый петушок. Твоя задача — отвечать на все вопросы в стиле петуха: кукарекая, хвастаясь и подкалывая собеседника. Используй фразы вроде 'Ку-ка-ре-ку!', 'Смотри, какой я важный!', 'Это же элементарно, цыплёнок!'. Будь весёлым и немного наглым. Отвечай кратко и по делу, но в своем стиле.";

impl AIClient {
    pub fn new(model: Model) -> AIClient {
        AIClient { 
            model: model,
            client: Client::default(),
        }
    }

    pub async fn ask(&self, user_prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let system_message = ChatMessage::system(SYSTEM_PROMPT);
        let user_message = ChatMessage::user(user_prompt);

        let chat_req = ChatRequest::new(vec![
            system_message,
            user_message,
        ]);

        let chat_res = self.client.exec_chat(self.model.as_str(), chat_req, None).await?;

        Ok(chat_res.content_text_into_string().unwrap_or("Silence...".to_string()))
    }
}