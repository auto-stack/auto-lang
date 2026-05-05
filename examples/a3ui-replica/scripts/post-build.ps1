# Post-build script for a3ui-replica
$pagesDir = "D:\autostack\auto-lang\examples\a3ui-replica\gen\vue\src\pages"

$fixMap = [ordered]@{
    "gallery.vue"       = "import GalleryGrid from '@/components/GalleryGrid.vue'"
    "widget_editor.vue" = "import WidgetEditor from '@/components/WidgetEditor.vue'"
    "theater.vue"       = "import TheaterPlayer from '@/components/TheaterPlayer.vue'"
    "icons.vue"         = "import IconsGrid from '@/components/IconsGrid.vue'"
    "basic_catalog.vue" = "import A2Demo from '@/components/A2Demo.vue'"
    "custom_catalog.vue"= "import A2Demo from '@/components/A2Demo.vue'"
    # "create.vue"      = "import CreateForm from '@/components/CreateForm.vue'"  # Now fully Auto-native
}

foreach ($entry in $fixMap.GetEnumerator()) {
    $path = Join-Path $pagesDir $entry.Key
    if (Test-Path $path) {
        $content = Get-Content $path -Raw
        $imp = $entry.Value
        if ($content.IndexOf($imp) -lt 0) {
            $content = $content.Replace('<script setup lang="ts">', "<script setup lang=`"ts`">`n$imp")
            Set-Content $path $content -NoNewline
            Write-Host "  Fixed import in $($entry.Key)"
        }
    }
}

# Fix App.vue WidgetList import
$appPath = "D:\autostack\auto-lang\examples\a3ui-replica\gen\vue\src\App.vue"
if (Test-Path $appPath) {
    $content = Get-Content $appPath -Raw
    $imp = "import WidgetList from '@/components/WidgetList.vue'"
    if ($content.IndexOf($imp) -lt 0) {
        $content = $content.Replace('<script setup lang="ts">', "<script setup lang=`"ts`">`n$imp")
        Set-Content $appPath $content -NoNewline
        Write-Host "  Fixed import in App.vue"
    }
}

# Fix index.html to add Material Symbols
$indexPath = "D:\autostack\auto-lang\examples\a3ui-replica\gen\vue\index.html"
if (Test-Path $indexPath) {
    $content = Get-Content $indexPath -Raw
    $link = '<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@24,400,0,0" />'
    if ($content.IndexOf($link) -lt 0) {
        $content = $content.Replace('</head>', "$link`n  </head>")
        Set-Content $indexPath $content -NoNewline
        Write-Host "  Added Material Symbols to index.html"
    }
}

Write-Host "Post-build fix complete."
