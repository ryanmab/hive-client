output "test_user_pool_id" {
  value = module.cognito.test_user_pool_id
}

output "test_user_pool_client_id" {
  value     = module.cognito.test_user_pool_client_id
  sensitive = true
}