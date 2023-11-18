# AlertmanagerExt

## Generate Models

```bash
curl -s https://raw.githubusercontent.com/prometheus/alertmanager/master/api/v2/openapi.yaml > api_v2_openapi.yaml

```

```PowerShell
Invoke-WebRequest -Uri https://raw.githubusercontent.com/prometheus/alertmanager/master/api/v2/openapi.yaml -OutFile api_v2_openapi.yaml
```

```bash
openapi-generator-cli generate -i api_v2_openapi.yaml -g rust -o ./alertmanager_api_v2
```
