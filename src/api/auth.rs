use crate::utils::{Result, YPlayerError};

const CLIENT_ID: &str = "23cabbbdc6cd418abb4b39c32c41195d";

pub struct AuthFlow;

impl AuthFlow {
    /// Ask user to get a token via browser OAuth and paste it.
    pub async fn get_token_interactive(&self) -> Result<String> {
        let auth_url = format!(
            "https://oauth.yandex.ru/authorize?response_type=token&client_id={}",
            CLIENT_ID
        );

        println!("\n🔑 Авторизация Яндекс Музыка\n");
        println!("1. Откроется браузер — войдите в Яндекc и разрешите доступ.");
        println!("2. После подтверждения вы будете перенаправлены на адрес вида:");
        println!("   https://localhost/?access_token=xxx...&token_type=bearer");
        println!("3. Скопируйте access_token из адресной строки и вставьте сюда.\n");

        // Try to open the browser
        if let Err(e) = open::that(&auth_url) {
            println!("Не удалось открыть браузер: {e}");
            println!("Откройте вручную: {auth_url}");
        } else {
            println!("Браузер открыт. Ожидаем ввод токена...");
        }

        let mut token = String::new();
        std::io::stdin()
            .read_line(&mut token)
            .map_err(|e| YPlayerError::Auth(format!("Ошибка ввода: {e}")))?;

        let token = token.trim().to_string();

        if token.is_empty() {
            return Err(YPlayerError::Auth("Токен не введён".into()));
        }

        // Verify the token works
        let client = reqwest::Client::new();
        let resp = client
            .get("https://api.music.yandex.net/account/status")
            .header("Authorization", format!("OAuth {token}"))
            .header("X-Yandex-Music-Client", "YandexMusicAndroid/24023621")
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(YPlayerError::Auth("Токен недействителен".into()));
        }

        Ok(token)
    }
}
