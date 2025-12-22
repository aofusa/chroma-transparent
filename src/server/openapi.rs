//! OpenAPIスキーマ生成

use serde_json::{json, Value};

/// OpenAPIスキーマを生成
pub fn generate_openapi_spec(video_enabled: bool) -> Value {
    let mut paths = json!({
        "/api/health": {
            "get": {
                "summary": "Health check",
                "operationId": "healthCheck",
                "tags": ["System"],
                "responses": {
                    "200": {
                        "description": "Server is healthy",
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/HealthResponse"
                                }
                            }
                        }
                    }
                }
            }
        },
        "/api/config": {
            "get": {
                "summary": "Get available parameters and their ranges",
                "operationId": "getConfig",
                "tags": ["Configuration"],
                "responses": {
                    "200": {
                        "description": "Parameter configuration",
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/ConfigResponse"
                                }
                            }
                        }
                    }
                }
            }
        },
        "/api/preview": {
            "post": {
                "summary": "Generate preview image (low resolution for fast response)",
                "operationId": "generatePreview",
                "tags": ["Processing"],
                "requestBody": {
                    "required": true,
                    "content": {
                        "multipart/form-data": {
                            "schema": {
                                "$ref": "#/components/schemas/PreviewRequest"
                            }
                        }
                    }
                },
                "responses": {
                    "200": {
                        "description": "Preview image",
                        "content": {
                            "image/png": {
                                "schema": {
                                    "type": "string",
                                    "format": "binary"
                                }
                            }
                        }
                    },
                    "400": {
                        "description": "Bad request",
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/ErrorResponse"
                                }
                            }
                        }
                    }
                }
            }
        },
        "/api/process": {
            "post": {
                "summary": "Process image with full quality",
                "operationId": "processImage",
                "tags": ["Processing"],
                "requestBody": {
                    "required": true,
                    "content": {
                        "multipart/form-data": {
                            "schema": {
                                "$ref": "#/components/schemas/ProcessRequest"
                            }
                        }
                    }
                },
                "responses": {
                    "200": {
                        "description": "Processing result",
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/ProcessResponse"
                                }
                            }
                        }
                    },
                    "400": {
                        "description": "Bad request",
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/ErrorResponse"
                                }
                            }
                        }
                    }
                }
            }
        },
        "/api/download/{file_id}": {
            "get": {
                "summary": "Download processed file",
                "operationId": "downloadFile",
                "tags": ["Files"],
                "parameters": [
                    {
                        "name": "file_id",
                        "in": "path",
                        "required": true,
                        "schema": {
                            "type": "string"
                        },
                        "description": "File ID returned from /api/process"
                    }
                ],
                "responses": {
                    "200": {
                        "description": "File download",
                        "content": {
                            "image/png": {
                                "schema": {
                                    "type": "string",
                                    "format": "binary"
                                }
                            },
                            "video/webm": {
                                "schema": {
                                    "type": "string",
                                    "format": "binary"
                                }
                            },
                            "video/quicktime": {
                                "schema": {
                                    "type": "string",
                                    "format": "binary"
                                }
                            }
                        }
                    },
                    "404": {
                        "description": "File not found",
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/ErrorResponse"
                                }
                            }
                        }
                    }
                }
            }
        },
        "/api/files/{file_id}": {
            "delete": {
                "summary": "Delete processed file",
                "operationId": "deleteFile",
                "tags": ["Files"],
                "parameters": [
                    {
                        "name": "file_id",
                        "in": "path",
                        "required": true,
                        "schema": {
                            "type": "string"
                        }
                    }
                ],
                "responses": {
                    "200": {
                        "description": "File deleted",
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/DeleteResponse"
                                }
                            }
                        }
                    },
                    "404": {
                        "description": "File not found"
                    }
                }
            }
        }
    });

    // video機能が有効な場合、動画処理エンドポイントを追加
    if video_enabled {
        paths.as_object_mut().unwrap().insert(
            "/api/process/video".to_string(),
            json!({
                "post": {
                    "summary": "Process video file (requires video feature)",
                    "operationId": "processVideo",
                    "tags": ["Processing", "Video"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "multipart/form-data": {
                                "schema": {
                                    "$ref": "#/components/schemas/VideoProcessRequest"
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Video processing result",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/ProcessResponse"
                                    }
                                }
                            }
                        },
                        "400": {
                            "description": "Bad request"
                        },
                        "501": {
                            "description": "Video processing not available"
                        }
                    }
                }
            }),
        );
    }

    json!({
        "openapi": "3.0.3",
        "info": {
            "title": "chroma-transparent API",
            "description": "Chroma key processing API for transparent image/video generation.\n\nThis API allows you to remove solid color backgrounds from images and videos, creating transparent PNG or WebM files.",
            "version": env!("CARGO_PKG_VERSION"),
            "license": {
                "name": "Apache-2.0",
                "url": "https://www.apache.org/licenses/LICENSE-2.0"
            }
        },
        "servers": [
            {
                "url": "/",
                "description": "Current server"
            }
        ],
        "tags": [
            {"name": "System", "description": "System endpoints"},
            {"name": "Configuration", "description": "Configuration endpoints"},
            {"name": "Processing", "description": "Image/Video processing endpoints"},
            {"name": "Files", "description": "File management endpoints"},
            {"name": "Video", "description": "Video-specific endpoints (requires --features video build and --enable-video runtime flag)"}
        ],
        "paths": paths,
        "components": {
            "schemas": generate_schemas(video_enabled)
        }
    })
}

/// スキーマ定義を生成
fn generate_schemas(video_enabled: bool) -> Value {
    let mut schemas = json!({
        "HealthResponse": {
            "type": "object",
            "properties": {
                "status": {"type": "string", "example": "ok"},
                "version": {"type": "string", "example": "0.1.0"},
                "video_enabled": {"type": "boolean", "description": "Whether video processing is available"}
            },
            "required": ["status", "version", "video_enabled"]
        },
        "ConfigResponse": {
            "type": "object",
            "properties": {
                "colors": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Available color names",
                    "example": ["lime", "green", "blue", "magenta", "cyan"]
                },
                "parameters": {
                    "type": "object",
                    "properties": {
                        "tolerance": {"$ref": "#/components/schemas/ParameterRange"},
                        "feather": {"$ref": "#/components/schemas/ParameterRange"},
                        "despill": {"$ref": "#/components/schemas/ParameterRange"},
                        "erode": {"$ref": "#/components/schemas/ParameterRange"},
                        "dilate": {"$ref": "#/components/schemas/ParameterRange"}
                    }
                },
                "video_enabled": {"type": "boolean"}
            }
        },
        "ParameterRange": {
            "type": "object",
            "properties": {
                "min": {"type": "number"},
                "max": {"type": "number"},
                "default": {"type": "number"},
                "step": {"type": "number"}
            },
            "required": ["min", "max", "default"]
        },
        "PreviewRequest": {
            "type": "object",
            "required": ["image"],
            "properties": {
                "image": {
                    "type": "string",
                    "format": "binary",
                    "description": "Image file to process"
                },
                "color": {
                    "type": "string",
                    "default": "lime",
                    "description": "Chroma key color (hex code or color name)"
                },
                "tolerance": {
                    "type": "number",
                    "default": 0.3,
                    "minimum": 0.0,
                    "maximum": 1.0,
                    "description": "Color tolerance"
                },
                "feather": {
                    "type": "integer",
                    "default": 5,
                    "minimum": 0,
                    "maximum": 50,
                    "description": "Edge feathering amount"
                },
                "despill": {
                    "type": "number",
                    "default": 0.7,
                    "minimum": 0.0,
                    "maximum": 1.0,
                    "description": "Color spill removal strength"
                },
                "erode": {
                    "type": "integer",
                    "default": 0,
                    "minimum": 0,
                    "maximum": 10,
                    "description": "Mask erosion iterations"
                },
                "dilate": {
                    "type": "integer",
                    "default": 1,
                    "minimum": 0,
                    "maximum": 10,
                    "description": "Mask dilation iterations"
                },
                "preview_size": {
                    "type": "integer",
                    "default": 512,
                    "description": "Maximum preview image dimension"
                }
            }
        },
        "ProcessRequest": {
            "type": "object",
            "required": ["image"],
            "properties": {
                "image": {
                    "type": "string",
                    "format": "binary",
                    "description": "Image file to process"
                },
                "color": {"type": "string", "default": "lime"},
                "tolerance": {"type": "number", "default": 0.3},
                "feather": {"type": "integer", "default": 5},
                "despill": {"type": "number", "default": 0.7},
                "erode": {"type": "integer", "default": 0},
                "dilate": {"type": "integer", "default": 1}
            }
        },
        "ProcessResponse": {
            "type": "object",
            "properties": {
                "success": {"type": "boolean"},
                "file_id": {"type": "string", "example": "20250623_143052_a1b2c3d4"},
                "download_url": {"type": "string", "example": "/api/download/20250623_143052_a1b2c3d4"},
                "filename": {"type": "string", "example": "20250623_143052_a1b2c3d4.png"},
                "processing_time_ms": {"type": "integer", "example": 234},
                "file_size": {"type": "integer", "example": 1234567}
            },
            "required": ["success", "file_id", "download_url", "filename"]
        },
        "DeleteResponse": {
            "type": "object",
            "properties": {
                "success": {"type": "boolean"},
                "file_id": {"type": "string"}
            }
        },
        "ErrorResponse": {
            "type": "object",
            "properties": {
                "error": {"type": "string", "example": "bad_request"},
                "message": {"type": "string", "example": "No image provided"}
            },
            "required": ["error", "message"]
        }
    });

    // video機能が有効な場合、動画用スキーマを追加
    if video_enabled {
        schemas.as_object_mut().unwrap().insert(
            "VideoProcessRequest".to_string(),
            json!({
                "type": "object",
                "required": ["video"],
                "properties": {
                    "video": {
                        "type": "string",
                        "format": "binary",
                        "description": "Video file to process"
                    },
                    "color": {"type": "string", "default": "lime"},
                    "tolerance": {"type": "number", "default": 0.3},
                    "feather": {"type": "integer", "default": 5},
                    "despill": {"type": "number", "default": 0.7},
                    "erode": {"type": "integer", "default": 0},
                    "dilate": {"type": "integer", "default": 1},
                    "video_format": {
                        "type": "string",
                        "enum": ["webm", "mov", "png-sequence"],
                        "default": "webm",
                        "description": "Output video format"
                    },
                    "video_quality": {
                        "type": "integer",
                        "default": 80,
                        "minimum": 1,
                        "maximum": 100,
                        "description": "Output video quality"
                    },
                    "fps": {
                        "type": "number",
                        "nullable": true,
                        "description": "Output frame rate (null = same as input)"
                    }
                }
            }),
        );
    }

    schemas
}

/// Swagger UI HTML を生成
pub fn swagger_ui_html() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>chroma-transparent API</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css">
    <style>
        html { box-sizing: border-box; overflow-y: scroll; }
        *, *:before, *:after { box-sizing: inherit; }
        body { margin: 0; background: #fafafa; }
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-standalone-preset.js"></script>
    <script>
        window.onload = () => {
            SwaggerUIBundle({
                url: "/api/openapi.json",
                dom_id: '#swagger-ui',
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIStandalonePreset
                ],
                plugins: [
                    SwaggerUIBundle.plugins.DownloadUrl
                ],
                layout: "StandaloneLayout"
            });
        };
    </script>
</body>
</html>"#
        .to_string()
}

