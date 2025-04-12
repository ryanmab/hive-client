output "test_user_pool_id" {
  value = aws_cognito_user_pool.user_pool.id
}

output "test_user_pool_client_id" {
  value     = aws_cognito_user_pool_client.test_client.id
  sensitive = true
}