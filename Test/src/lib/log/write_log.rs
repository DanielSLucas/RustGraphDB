use std::fs::File;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::path::Path;

#[derive(Clone)]
pub struct TextLogger {
    data: Arc<Mutex<String>>, // Guarda o texto de maneira compartilhada e segura
    output_file: String,
}

impl TextLogger {
    // Cria uma nova instância do logger com o caminho do arquivo de saída
    pub fn new(output_file:String) -> Self {
        TextLogger {
            data: Arc::new(Mutex::new(String::new())),
            output_file: output_file.to_string(),
        }
    }

    // Método para adicionar texto ao logger
    pub async fn log(&self, text: String) {
        let mut data = self.data.lock().unwrap(); // Bloqueia o acesso ao dado
        data.push_str(&text); // Adiciona o texto
        data.push('\n'); // Adiciona uma nova linha para cada entrada de log
    }

    // Método para gerar o arquivo com o conteúdo registrado
    pub async fn write_to_file(&self) -> io::Result<()> {
        let path = Path::new(&self.output_file);
        let mut file = File::create(&path)?;
        let data = self.data.lock().unwrap();
        file.write_all(data.as_bytes())?;
        Ok(())
    }
}