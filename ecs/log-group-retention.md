# Research App Log Group

CloudWatch Logs retention must be 3 days.

Approved retention:

```text
retentionInDays = 3
```

Approved log group name:

```text
/aws/ecs/log-nangman-dev-research-apn2
```

Creation or update command must be executed only after resource-name approval:

```bash
aws logs create-log-group \
  --region ap-northeast-2 \
  --log-group-name /aws/ecs/log-nangman-dev-research-apn2 \
  --tags Name=/aws/ecs/log-nangman-dev-research-apn2,Environment=dev,Scope=shared,Owner=seongwon

aws logs put-retention-policy \
  --region ap-northeast-2 \
  --log-group-name /aws/ecs/log-nangman-dev-research-apn2 \
  --retention-in-days 3
```
