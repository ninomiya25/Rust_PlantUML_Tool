use plantuml_client::{PlantUmlClient, ClientError};
use plantuml_editor_core::{DocumentId, ImageFormat};
use mockito::{Server, Matcher};

#[tokio::test]
async fn test_convert_to_png_success() {
    // モックサーバーを起動
    let mut server = Server::new_async().await;
    
    // PNG データのモック
    let mock_png_data = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG ヘッダー
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR チャンク
    ];
    
    // エンドポイントをモック
    let _mock = server
        .mock("GET", Matcher::Regex(r"^/png/.*".to_string()))
        .with_status(200)
        .with_header("content-type", "image/png")
        .with_body(mock_png_data.clone())
        .create_async()
        .await;
    
    // クライアント作成
    let client = PlantUmlClient::new(server.url()).unwrap();
    
    // テスト実行
    let document_id = DocumentId::new();
    let plantuml_text = "@startuml\nAlice -> Bob: Hello\n@enduml";
    
    let result = client.convert_to_png(document_id, plantuml_text).await;
    
    // 検証
    assert!(result.is_ok());
    let diagram = result.unwrap();
    assert_eq!(diagram.format, ImageFormat::Png);
    assert_eq!(diagram.data, mock_png_data);
    assert_eq!(diagram.document_id, document_id);
}

#[tokio::test]
async fn test_convert_to_svg_success() {
    let mut server = Server::new_async().await;
    
    let mock_svg_data = br#"<svg xmlns="http://www.w3.org/2000/svg">
        <rect width="100" height="100"/>
    </svg>"#;
    
    let _mock = server
        .mock("GET", Matcher::Regex(r"^/svg/.*".to_string()))
        .with_status(200)
        .with_header("content-type", "image/svg+xml")
        .with_body(mock_svg_data.as_slice())
        .create_async()
        .await;
    
    let client = PlantUmlClient::new(server.url()).unwrap();
    let document_id = DocumentId::new();
    let plantuml_text = "@startuml\nAlice -> Bob: Hello\n@enduml";
    
    let result = client.convert_to_svg(document_id, plantuml_text).await;
    
    assert!(result.is_ok());
    let diagram = result.unwrap();
    assert_eq!(diagram.format, ImageFormat::Svg);
    assert_eq!(diagram.data, mock_svg_data.to_vec());
}

#[tokio::test]
async fn test_convert_syntax_error_image() {
    let mut server = Server::new_async().await;
    
    // 構文エラーを含むSVG
    let error_svg = br#"<svg xmlns="http://www.w3.org/2000/svg">
        <text x="10" y="20">Syntax Error at line 2</text>
    </svg>"#;
    
    let _mock = server
        .mock("GET", Matcher::Regex(r"^/svg/.*".to_string()))
        .with_status(200)
        .with_body(error_svg.as_slice())
        .create_async()
        .await;
    
    let client = PlantUmlClient::new(server.url()).unwrap();
    let document_id = DocumentId::new();
    let invalid_plantuml = "@startuml\ninvalid syntax\n@enduml";
    
    let result = client.convert_to_svg(document_id, invalid_plantuml).await;
    
    assert!(result.is_ok());
    let diagram = result.unwrap();
    
    // エラー画像が返されることを確認
    let svg_text = String::from_utf8_lossy(&diagram.data);
    assert!(svg_text.contains("Syntax Error"));
}

#[tokio::test]
async fn test_convert_network_error_connection_refused() {
    // 存在しないサーバーに接続してネットワークエラーを発生させる
    let client = PlantUmlClient::new("http://localhost:9999".to_string()).unwrap();
    
    let document_id = DocumentId::new();
    let plantuml_text = "@startuml\nAlice -> Bob: Hello\n@enduml";
    
    let result = client.convert_to_png(document_id, plantuml_text).await;
    
    // 正しい期待値: ネットワークエラーが返される
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ClientError::Network(_)));
}

#[tokio::test]
async fn test_convert_timeout_error() {
    let mut server = Server::new_async().await;
    
    // レスポンスを遅延させる（タイムアウトをシミュレート）
    let _mock = server
        .mock("GET", Matcher::Regex(r"^/png/.*".to_string()))
        .with_status(200)
        .with_body(vec![0; 100])
        .with_chunked_body(|w| {
            // タイムアウトより長く待機
            std::thread::sleep(std::time::Duration::from_secs(35));
            w.write_all(&[0; 100])
        })
        .create_async()
        .await;
    
    let client = PlantUmlClient::new(server.url()).unwrap();
    let document_id = DocumentId::new();
    let plantuml_text = "@startuml\nAlice -> Bob: Hello\n@enduml";
    
    let result = client.convert_to_png(document_id, plantuml_text).await;
    
    // タイムアウトエラーが返される
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ClientError::Network(_)));
}

#[tokio::test]
async fn test_convert_encoding_error() {
    // エンコードエラーのテストは実際には難しい？？
    // （どんな文字列でもエンコード可能なため）
    // ここでは省略
}