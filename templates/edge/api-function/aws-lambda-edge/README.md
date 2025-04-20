# {{project_name}} - Rust AWS Lambda@Edge Function

This project is a Rust application built for AWS Lambda@Edge. It leverages Rust's performance and safety features to deliver high-performance edge computing functions that can be deployed globally via Amazon CloudFront.

## 📋 Features

- ⚡️ High-performance edge function powered by Rust
- 🌐 Global deployment across AWS's edge network via CloudFront
- 🧩 Flexible API routing with path and query parameter support
- 🔄 JSON serialization and deserialization
- 🔍 Request header processing
- 📊 AWS SAM template for easy deployment

## 🚀 Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:
- [Rust](https://www.rust-lang.org/tools/install)
- [AWS CLI](https://aws.amazon.com/cli/)
- [AWS SAM CLI](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-cli-install.html)
- AWS Account with appropriate permissions

### Development

1. Install the AWS SAM CLI if you haven't already:
   ```
   brew install aws/tap/aws-sam-cli
   ```

2. Configure your AWS credentials:
   ```
   aws configure
   ```

3. Build the project:
   ```
   sam build
   ```

4. Start the local development server:
   ```
   sam local start-api
   ```

5. Your API will be available at `http://localhost:3000`

### Deployment

1. Build the project for deployment:
   ```
   sam build
   ```

2. Deploy to AWS Lambda:
   ```
   sam deploy --guided
   ```
   
   This will walk you through the deployment process, prompting for:
   - Stack name
   - AWS Region
   - Parameter values
   - Confirmation of IAM role creation

3. To update an existing deployment:
   ```
   sam deploy
   ```

## 📖 API Documentation

### Available Endpoints

- `GET /` - Returns a simple HTML page
- `GET /api` - Returns a JSON response with a greeting message
- `GET /api/echo?message=your-message` - Echo back the provided message as JSON
- `GET /api/headers` - Returns the request headers as a JSON response

### Example Requests

```bash
# Get the default API response
curl -X GET "https://your-api-gateway-url.execute-api.region.amazonaws.com/prod/api"

# Echo a message
curl -X GET "https://your-api-gateway-url.execute-api.region.amazonaws.com/prod/api/echo?message=hello-world"

# Get request headers
curl -X GET "https://your-api-gateway-url.execute-api.region.amazonaws.com/prod/api/headers"
```

## 🔧 Customization

### Adding New Routes

Modify the `handler` function in `src/main.rs` to add new routes:

```rust
match (method, path) {
    // Existing routes...
    
    // Add your new route here
    ("GET", "/api/new-endpoint") => {
        // Your handler code
        let response_data = ApiResponse {
            message: "Your new endpoint response".to_string(),
            status: "success".to_string(),
            timestamp: current_timestamp(),
            path: Some("/api/new-endpoint".to_string()),
            method: Some("GET".to_string()),
            headers: None,
            query_parameters: None,
        };
        
        // Serialize and return the JSON response
        let json = serde_json::to_string(&response_data)?;
        let response = Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/json")
            .body(Body::from(json))?;
        
        Ok(response)
    },
    
    // Default 404 handler
    _ => { /* ... */ }
}
```

### Modifying the SAM Template

The `template.yaml` file contains the AWS SAM configuration. You can modify this to:

1. Change the memory allocation or timeout:
   ```yaml
   Globals:
     Function:
       Timeout: 10  # Increase timeout
       MemorySize: 256  # Increase memory
   ```

2. Add CloudFront distribution for true Lambda@Edge (uncomment the EdgeDistribution section)

3. Add custom domain names, additional resources, etc.

## 📚 Advanced Features

### True Lambda@Edge Deployment

The template includes commented sections for deploying as a true Lambda@Edge function via CloudFront. To use this:

1. Uncomment the `EdgeDistribution` section in `template.yaml`
2. Update the origin domain name to your actual origin server
3. Deploy with `sam deploy --guided`

### Environment Variables

Lambda@Edge has restrictions on environment variables. For configuration:

1. For API Gateway + Lambda: Use regular environment variables in the SAM template
2. For true Lambda@Edge: Use SSM Parameter Store or embed configuration in the function code

### CloudWatch Logs

The function automatically logs information using the tracing module:

```rust
tracing::info!("Custom log message with data: {}", some_value);
```

Access these logs in the CloudWatch console or with:
```
aws logs get-log-events --log-group-name /aws/lambda/{{project_name}} --log-stream-name latest
```

## 📊 Monitoring and Scaling

### CloudWatch Metrics

Monitor your function's performance with CloudWatch metrics:

1. Invocation count and duration
2. Error rates and throttling
3. Cold start frequency

### Auto Scaling

Lambda functions automatically scale based on demand. No configuration is needed, but consider:

1. Setting appropriate memory size for performance
2. Using Provisioned Concurrency for critical workloads to avoid cold starts

## 📖 Resources

- [AWS Lambda@Edge Documentation](https://docs.aws.amazon.com/lambda/latest/dg/lambda-edge.html)
- [AWS SAM Documentation](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/what-is-sam.html)
- [Rust AWS Lambda Runtime](https://github.com/awslabs/aws-lambda-rust-runtime)
