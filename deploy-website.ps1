$server = "visus@112.74.45.241"
$remotePath = "/home/visus/auto-website"

Set-Location "$PSScriptRoot\website"
npm run build
if ($LASTEXITCODE -ne 0) { throw "构建失败" }

$localDist = "$PSScriptRoot\website\.vitepress\dist"
$tempTar = "$env:TEMP\auto-website-dist.tar.gz"

tar -czf $tempTar -C $localDist .
scp $tempTar "${server}:/tmp/auto-website-dist.tar.gz"

ssh $server "rm -rf $remotePath/* && tar -xzf /tmp/auto-website-dist.tar.gz -C $remotePath && rm /tmp/auto-website-dist.tar.gz"

Remove-Item $tempTar
Write-Host "部署完成！请访问 http://112.74.45.241"
