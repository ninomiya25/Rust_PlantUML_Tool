use plantuml_editor_api_client::{convert_plantuml, export_plantuml};
use plantuml_editor_core::{ErrorCode, ImageFormat, StatusLevel};
use serde_json::json;
use serial_test::serial;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

// ========================================
// テスト用ヘルパー関数
// ========================================

/// テスト環境でプロキシを無効化
fn disable_proxy_for_test() {
    // プロキシ環境変数を削除してlocalhostへのアクセスを許可
    std::env::remove_var("HTTP_PROXY");
    std::env::remove_var("HTTPS_PROXY");
    std::env::remove_var("http_proxy");
    std::env::remove_var("https_proxy");
    // localhostをNO_PROXYに追加
    std::env::set_var("NO_PROXY", "localhost,127.0.0.1");
}

// ========================================
// プロキシ無効化テスト用のヘルパー
// ========================================

#[tokio::test]
#[serial]
async fn test_proxy_disabled() {
    disable_proxy_for_test();
    
    // プロキシを無効化したreqwestクライアントでシンプルなテスト
    let mut server = mockito::Server::new_async().await;
    
    let _mock = server
        .mock("GET", "/test")
        .with_status(200)
        .with_body("hello")
        .create_async()
        .await;
    
    // プロキシを無効化したクライアントを作成
    let client = reqwest::Client::builder()
        .no_proxy()
        .build()
        .unwrap();
    
    let response = client
        .get(format!("{}/test", server.url()))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    let body = response.text().await.unwrap();
    assert_eq!(body, "hello");
}

// ========================================
// convert_plantuml のテスト
// ========================================

#[tokio::test]
#[serial]
async fn test_convert_plantuml_success() {
    disable_proxy_for_test();
    
    // 1. mockitoサーバーを起動
    let mut server = mockito::Server::new_async().await;
    
    // 2. API_BASE_URLを設定
    std::env::set_var("API_BASE_URL", server.url());
    
    // 3. モックレスポンスを定義（バイナリデータを配列としてJSONに含める）
    let mock_response = json!({
        "result": {
            "level": "INFO",
            "code": {
                "type": "ConversionOk"
            }
        },
        "image_data": [137, 80, 78, 71] // PNG magic bytes as array
    });
    
    // 4. モックエンドポイントを登録
    let mock = server
        .mock("POST", "/api/v1/convert")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create_async()
        .await;
    
    // 5. テスト対象の関数を実行
    let result = convert_plantuml(
        "@startuml\nAlice -> Bob\n@enduml".to_string(),
        ImageFormat::Svg,
    )
    .await;
    
    // 6. アサーション
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result);
    let (image_data, process_result) = result.unwrap();
    
    assert_eq!(image_data, vec![137, 80, 78, 71]);
    assert_eq!(process_result.level, StatusLevel::Info);
    assert!(matches!(process_result.code, ErrorCode::ConversionOk));
    
    // Mock was called
    mock.assert_async().await;
}


#[tokio::test]
#[serial]
async fn test_convert_plantuml_network_error() {
    disable_proxy_for_test();
    
    // モックサーバーを起動しない（接続失敗をシミュレート）
    std::env::set_var("API_BASE_URL", "http://localhost:9999");
    
    let result = convert_plantuml(
        "@startuml\nAlice -> Bob\n@enduml".to_string(),
        ImageFormat::Svg,
    )
    .await;
    
    assert!(result.is_err());
    if let Err(plantuml_editor_api_client::ApiError::NetworkError(msg)) = result {
        assert!(msg.contains("サーバーが応答していません"));
    } else {
        panic!("Expected NetworkError");
    }
}

#[tokio::test]
#[serial]
async fn test_convert_plantuml_validation_error() {
    disable_proxy_for_test();
    
    let mock_server = MockServer::start().await;
    std::env::set_var("API_BASE_URL", mock_server.uri());
    
    // バリデーションエラーのレスポンス
    let mock_response = json!({
        "result": {
            "level": "WARNING",
            "code": {
                "type": "ValidationEmpty"
            }
        },
        "image_data": null
    });
    
    Mock::given(method("POST"))
        .and(path("/api/v1/convert"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
        .mount(&mock_server)
        .await;
    
    let result = convert_plantuml(
        "".to_string(),
        ImageFormat::Svg,
    )
    .await;
    
    assert!(result.is_err());
    if let Err(plantuml_editor_api_client::ApiError::ProcessError(error_code)) = result {
        assert_eq!(error_code.status_level(), StatusLevel::Warning);
        assert!(matches!(error_code, ErrorCode::ValidationEmpty));
    } else {
        panic!("Expected ProcessError");
    }
}

#[tokio::test]
#[serial]
async fn test_convert_plantuml_http_500_error() {
    disable_proxy_for_test();
    
    let mock_server = MockServer::start().await;
    std::env::set_var("API_BASE_URL", mock_server.uri());
    
    // HTTP 500エラーをシミュレート
    Mock::given(method("POST"))
        .and(path("/api/v1/convert"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;
    
    let result = convert_plantuml(
        "@startuml\nAlice -> Bob\n@enduml".to_string(),
        ImageFormat::Svg,
    )
    .await;
    
    assert!(result.is_err());
    if let Err(plantuml_editor_api_client::ApiError::ServerError(msg)) = result {
        assert!(msg.contains("HTTPエラー: 500"));
    } else {
        panic!("Expected ServerError");
    }
}

#[tokio::test]
#[serial]
async fn test_convert_plantuml_invalid_json_response() {
    disable_proxy_for_test();
    
    let mock_server = MockServer::start().await;
    std::env::set_var("API_BASE_URL", mock_server.uri());
    
    // 無効なJSONレスポンス
    Mock::given(method("POST"))
        .and(path("/api/v1/convert"))
        .respond_with(ResponseTemplate::new(200).set_body_string("invalid json"))
        .mount(&mock_server)
        .await;
    
    let result = convert_plantuml(
        "@startuml\nAlice -> Bob\n@enduml".to_string(),
        ImageFormat::Svg,
    )
    .await;
    
    assert!(result.is_err());
    if let Err(plantuml_editor_api_client::ApiError::NetworkError(msg)) = result {
        assert!(msg.contains("レスポンスの解析に失敗しました"));
    } else {
        panic!("Expected NetworkError");
    }
}

// ========================================
// export_plantuml のテスト
// ========================================

#[tokio::test]
#[serial]
async fn test_export_plantuml_success() {
    disable_proxy_for_test();
    
    let mock_server = MockServer::start().await;
    std::env::set_var("API_BASE_URL", mock_server.uri());
    
    let mock_response = json!({
        "result": {
            "level": "INFO",
            "code": {
                "type": "ConversionOk"
            }
        },
        "image_data": vec![0xFF, 0xD8, 0xFF, 0xE0] // JPEG magic bytes
    });
    
    Mock::given(method("POST"))
        .and(path("/api/v1/export"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
        .mount(&mock_server)
        .await;
    
    let result = export_plantuml(
        "@startuml\nAlice -> Bob\n@enduml".to_string(),
        ImageFormat::Png,
    )
    .await;
    
    assert!(result.is_ok());
    let (image_data, process_result) = result.unwrap();
    
    assert_eq!(image_data, vec![0xFF, 0xD8, 0xFF, 0xE0]);
    assert_eq!(process_result.level, StatusLevel::Info);
}

#[tokio::test]
#[serial]
async fn test_export_plantuml_parse_error() {
    disable_proxy_for_test();
    
    let mock_server = MockServer::start().await;
    std::env::set_var("API_BASE_URL", mock_server.uri());
    
    let mock_response = json!({
        "result": {
            "level": "ERROR",
            "code": {
                "type": "ParseError",
                "line": 3
            }
        },
        "image_data": null
    });
    
    Mock::given(method("POST"))
        .and(path("/api/v1/export"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
        .mount(&mock_server)
        .await;
    
    let result = export_plantuml(
        "@startuml\ninvalid syntax\n@enduml".to_string(),
        ImageFormat::Png,
    )
    .await;
    
    assert!(result.is_err());
    if let Err(plantuml_editor_api_client::ApiError::ProcessError(error_code)) = result {
        assert_eq!(error_code.status_level(), StatusLevel::Error);
        if let ErrorCode::ParseError { line } = error_code {
            assert_eq!(line, Some(3));
        } else {
            panic!("Expected ParseError");
        }
    } else {
        panic!("Expected ProcessError");
    }
}
