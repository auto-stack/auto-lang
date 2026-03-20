//! ArkTS Project Generator
//!
//! Generates complete HarmonyOS project structure for ArkTS applications.

use std::collections::HashMap;

/// ArkTS project generator
pub struct ArkProjectGenerator {
    /// Project name
    pub name: String,
    /// Package name (e.g., "com.example.myapp")
    pub package: String,
}

impl ArkProjectGenerator {
    /// Create a new project generator
    pub fn new(name: &str) -> Self {
        let package = format!("com.example.{}", name.to_lowercase().replace('-', "_"));
        Self {
            name: name.to_string(),
            package,
        }
    }

    /// Create with custom package
    pub fn with_package(name: &str, package: &str) -> Self {
        Self {
            name: name.to_string(),
            package: package.to_string(),
        }
    }

    /// Generate all project files
    pub fn generate(&self) -> HashMap<String, String> {
        let mut files = HashMap::new();

        // Root level files
        files.insert(
            "build-profile.json5".to_string(),
            self.generate_build_profile(),
        );

        files.insert(
            "oh-package.json5".to_string(),
            self.generate_oh_package(),
        );

        files.insert(
            "hvigorfile.ts".to_string(),
            self.generate_hvigorfile(),
        );

        files.insert(
            "code-linter.json5".to_string(),
            self.generate_code_linter(),
        );

        // AppScope directory
        files.insert(
            "AppScope/app.json5".to_string(),
            self.generate_app_json5(),
        );

        files.insert(
            "AppScope/resources/base/element/string.json".to_string(),
            self.generate_app_strings(),
        );

        // AppScope media resources
        files.insert(
            "AppScope/resources/base/media/layered_image.json".to_string(),
            self.generate_layered_image(),
        );

        files.insert(
            "AppScope/resources/base/media/background.png".to_string(),
            self.generate_placeholder_icon("background"),
        );

        files.insert(
            "AppScope/resources/base/media/foreground.png".to_string(),
            self.generate_placeholder_icon("foreground"),
        );

        // Hvigor build system
        files.insert(
            "hvigor/hvigor-config.json5".to_string(),
            self.generate_hvigor_config(),
        );

        // Entry module
        files.insert(
            "entry/build-profile.json5".to_string(),
            self.generate_entry_build_profile(),
        );

        files.insert(
            "entry/hvigorfile.ts".to_string(),
            self.generate_entry_hvigorfile(),
        );

        files.insert(
            "entry/oh-package.json5".to_string(),
            self.generate_entry_oh_package(),
        );

        files.insert(
            "entry/obfuscation-rules.txt".to_string(),
            self.generate_obfuscation_rules(),
        );

        files.insert(
            "entry/src/main/module.json5".to_string(),
            self.generate_module_json5(),
        );

        // Entry resources
        files.insert(
            "entry/src/main/resources/base/element/color.json".to_string(),
            self.generate_colors(),
        );

        files.insert(
            "entry/src/main/resources/base/element/string.json".to_string(),
            self.generate_entry_strings(),
        );

        files.insert(
            "entry/src/main/resources/base/element/float.json".to_string(),
            self.generate_floats(),
        );

        files.insert(
            "entry/src/main/resources/base/profile/main_pages.json".to_string(),
            self.generate_main_pages(),
        );

        // Entry media resources
        files.insert(
            "entry/src/main/resources/base/media/layered_image.json".to_string(),
            self.generate_layered_image(),
        );

        files.insert(
            "entry/src/main/resources/base/media/background.png".to_string(),
            self.generate_placeholder_icon("background"),
        );

        files.insert(
            "entry/src/main/resources/base/media/foreground.png".to_string(),
            self.generate_placeholder_icon("foreground"),
        );

        files.insert(
            "entry/src/main/resources/base/media/startIcon.png".to_string(),
            self.generate_placeholder_icon("startIcon"),
        );

        // Entry ability
        files.insert(
            "entry/src/main/ets/entryability/EntryAbility.ets".to_string(),
            self.generate_entry_ability(),
        );

        // Main page
        files.insert(
            "entry/src/main/ets/pages/Index.ets".to_string(),
            self.generate_index_page(),
        );

        files
    }

    /// Generate build-profile.json5 (root)
    fn generate_build_profile(&self) -> String {
        format!(
            r#"{{
  "app": {{
    "signingConfigs": [],
    "products": [
      {{
        "name": "default",
        "signingConfig": "default",
        "targetSdkVersion": "6.0.2(22)",
        "compatibleSdkVersion": "6.0.2(22)",
        "runtimeOS": "HarmonyOS",
        "buildOption": {{
          "strictMode": {{
            "caseSensitiveCheck": true,
            "useNormalizedOHMUrl": true
          }}
        }}
      }}
    ],
    "buildModeSet": [
      {{
        "name": "debug"
      }},
      {{
        "name": "release"
      }}
    ]
  }},
  "modules": [
    {{
      "name": "entry",
      "srcPath": "./entry",
      "targets": [
        {{
          "name": "default",
          "applyToProducts": [
            "default"
          ]
        }}
      ]
    }}
  ]
}}"#
        )
    }

    /// Generate oh-package.json5 (root)
    fn generate_oh_package(&self) -> String {
        format!(
            r#"{{
  "modelVersion": "6.0.2",
  "description": "{} - A HarmonyOS application generated by AutoLang",
  "dependencies": {{}},
  "devDependencies": {{
    "@ohos/hypium": "1.0.25",
    "@ohos/hamock": "1.0.0"
  }}
}}"#,
            self.name
        )
    }

    /// Generate hvigorfile.ts (root)
    fn generate_hvigorfile(&self) -> String {
        r#"import { appTasks } from '@ohos/hvigor-ohos-plugin';

export default {
  system: appTasks, /* Built-in plugin of Hvigor. It cannot be modified. */
  plugins: []       /* Custom plugin to extend the functionality of Hvigor. */
}
"#.to_string()
    }

    /// Generate code-linter.json5
    fn generate_code_linter(&self) -> String {
        r#"{
  "file": [
    "**/*.ets"
  ],
  "ignore": [
    "**/oh_modules/**",
    "**/node_modules/**"
  ],
  "ruleSet": [
    "plugin:@typescript-eslint/recommended"
  ],
  "rules": {
  }
}
"#.to_string()
    }

    /// Generate AppScope/app.json5
    fn generate_app_json5(&self) -> String {
        format!(
            r#"{{
  "app": {{
    "bundleName": "{}",
    "vendor": "example",
    "versionCode": 1000000,
    "versionName": "1.0.0",
    "icon": "$media:layered_image",
    "label": "$string:app_name"
  }}
}}"#,
            self.package
        )
    }

    /// Generate AppScope strings
    fn generate_app_strings(&self) -> String {
        format!(
            r#"{{
  "string": [
    {{
      "name": "app_name",
      "value": "{}"
    }}
  ]
}}"#,
            self.name
        )
    }

    /// Generate hvigor/hvigor-config.json5
    fn generate_hvigor_config(&self) -> String {
        r#"{
  "modelVersion": "6.0.2",
  "dependencies": {
  },
  "execution": {
  },
  "logging": {
  },
  "debugging": {
  },
  "nodeOptions": {
  }
}
"#.to_string()
    }

    /// Generate entry/build-profile.json5
    fn generate_entry_build_profile(&self) -> String {
        r#"{
  "apiType": "stageMode",
  "buildOption": {
  },
  "buildOptionSet": [
    {
      "name": "release",
      "arkOptions": {
        "obfuscation": {
          "ruleOptions": {
            "enable": true
          }
        }
      }
    }
  ],
  "targets": [
    {
      "name": "default"
    }
  ]
}
"#.to_string()
    }

    /// Generate entry/hvigorfile.ts
    fn generate_entry_hvigorfile(&self) -> String {
        r#"import { hapTasks } from '@ohos/hvigor-ohos-plugin';

export default {
  system: hapTasks,  /* Built-in plugin of Hvigor. It cannot be modified. */
  plugins: []        /* Custom plugin to extend the functionality of Hvigor. */
}
"#.to_string()
    }

    /// Generate entry/oh-package.json5
    fn generate_entry_oh_package(&self) -> String {
        r#"{
  "name": "entry",
  "version": "1.0.0",
  "description": "Entry module",
  "main": "",
  "author": "",
  "license": "MIT",
  "dependencies": {}
}
"#.to_string()
    }

    /// Generate obfuscation-rules.txt
    fn generate_obfuscation_rules(&self) -> String {
        r#"# Define obfuscation rules for your project.
# For more details, see:
#   https://developer.huawei.com/consumer/cn/doc/harmonyos-guides-V5/ide-obfuscation-V5
"#.to_string()
    }

    /// Generate module.json5
    fn generate_module_json5(&self) -> String {
        format!(
            r#"{{
  "module": {{
    "name": "entry",
    "type": "entry",
    "description": "$string:module_desc",
    "mainElement": "EntryAbility",
    "deviceTypes": [
      "phone"
    ],
    "deliveryWithInstall": true,
    "installationFree": false,
    "pages": "$profile:main_pages",
    "abilities": [
      {{
        "name": "EntryAbility",
        "srcEntry": "./ets/entryability/EntryAbility.ets",
        "description": "$string:EntryAbility_desc",
        "icon": "$media:layered_image",
        "label": "$string:EntryAbility_label",
        "startWindowIcon": "$media:startIcon",
        "startWindowBackground": "$color:start_window_background",
        "exported": true,
        "skills": [
          {{
            "entities": [
              "entity.system.home"
            ],
            "actions": [
              "ohos.want.action.home"
            ]
          }}
        ]
      }}
    ]
  }}
}}"#
        )
    }

    /// Generate entry colors
    fn generate_colors(&self) -> String {
        r##"{
  "color": [
    {
      "name": "start_window_background",
      "value": "#FFFFFF"
    }
  ]
}
"##.to_string()
    }

    /// Generate entry strings
    fn generate_entry_strings(&self) -> String {
        format!(
            r#"{{
  "string": [
    {{
      "name": "module_desc",
      "value": "Entry module description"
    }},
    {{
      "name": "EntryAbility_desc",
      "value": "Main entry ability"
    }},
    {{
      "name": "EntryAbility_label",
      "value": "{}"
    }}
  ]
}}"#,
            self.name
        )
    }

    /// Generate entry floats
    fn generate_floats(&self) -> String {
        r#"{
  "float": [
    {
      "name": "page_text_font_size",
      "value": "50fp"
    }
  ]
}
"#.to_string()
    }

    /// Generate layered_image.json for adaptive icon
    fn generate_layered_image(&self) -> String {
        r#"{
  "layered-image":
  {
    "background" : "$media:background",
    "foreground" : "$media:foreground"
  }
}
"#.to_string()
    }

    /// Generate a placeholder PNG icon (base64 encoded 48x48 purple square)
    /// This is a minimal placeholder - replace with actual icons for production
    fn generate_placeholder_icon(&self, _name: &str) -> String {
        // Base64 encoded 48x48 PNG with a simple purple gradient
        // This is a placeholder that should be replaced with actual app icons
        "iVBORw0KGgoAAAANSUhEUgAAADAAAAAwCAYAAABXAvmHAAAACXBIWXMAAAsTAAALEwEAmpwYAAAB\
        hklEQVR4nO2ZsU7DMBCGP0QH6Ao4FRQcCjqCDgVFDqKj4CCo4KBig0EHQUEFHI1LABK7idZk2vF3\
        +Jv5S5Zl+Uhy/8vy/KQhSVKCJGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhS\
        giRlSFKCJGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhSgiRlSFKCJGVIUoIk\
        ZUhSgiRlSFKCJGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhSgiRlSFKCJGVI\
        UoIkZUhSgiRlSFKCJGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhSgiRlSFKC\
        JGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhSgiRl\
        SFKCJGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhS\
        giRlSFKCJGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhSgiRlSFKCJGVIUoIk\
        ZUhSgiRlSFKCJGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhSgiRlSFKCJGVI\
        UoIkZUhSgiRlSFKCJGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhSgiRlSFKC\
        JGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhSgiRl\
        SFKCJGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhSgiRlSFKCJGVIUoIkZUhS\
        giRlSFKCJGVIUoIkZUhSgiT1AeLcB9YB4gMAAAAASUVORK5CYII=".to_string()
    }

    /// Generate main_pages.json
    fn generate_main_pages(&self) -> String {
        r#"{
  "src": [
    "pages/Index"
  ]
}
"#.to_string()
    }

    /// Generate EntryAbility.ets
    fn generate_entry_ability(&self) -> String {
        r#"import { AbilityConstant, ConfigurationConstant, UIAbility, Want } from '@kit.AbilityKit';
import { hilog } from '@kit.PerformanceAnalysisKit';
import { window } from '@kit.ArkUI';

const DOMAIN = 0x0000;

export default class EntryAbility extends UIAbility {
  onCreate(want: Want, launchParam: AbilityConstant.LaunchParam): void {
    try {
      this.context.getApplicationContext().setColorMode(ConfigurationConstant.ColorMode.COLOR_MODE_NOT_SET);
    } catch (err) {
      hilog.error(DOMAIN, 'EntryAbility', 'Failed to set colorMode. Cause: %{public}s', JSON.stringify(err));
    }
    hilog.info(DOMAIN, 'EntryAbility', '%{public}s', 'Ability onCreate');
  }

  onDestroy(): void {
    hilog.info(DOMAIN, 'EntryAbility', '%{public}s', 'Ability onDestroy');
  }

  onWindowStageCreate(windowStage: window.WindowStage): void {
    hilog.info(DOMAIN, 'EntryAbility', '%{public}s', 'Ability onWindowStageCreate');

    windowStage.loadContent('pages/Index', (err) => {
      if (err.code) {
        hilog.error(DOMAIN, 'EntryAbility', 'Failed to load the content. Cause: %{public}s', JSON.stringify(err));
        return;
      }
      hilog.info(DOMAIN, 'EntryAbility', 'Succeeded in loading the content.');
    });
  }

  onWindowStageDestroy(): void {
    hilog.info(DOMAIN, 'EntryAbility', '%{public}s', 'Ability onWindowStageDestroy');
  }

  onForeground(): void {
    hilog.info(DOMAIN, 'EntryAbility', '%{public}s', 'Ability onForeground');
  }

  onBackground(): void {
    hilog.info(DOMAIN, 'EntryAbility', '%{public}s', 'Ability onBackground');
  }
}
"#.to_string()
    }

    /// Generate Index.ets main page
    fn generate_index_page(&self) -> String {
        format!(
            r#"// Generated by AutoLang ArkTS Generator
// {}

@Entry
@Component
struct Index {{
  @State message: string = 'Hello HarmonyOS'

  build() {{
    Column() {{
      Text(this.message)
        .fontSize(24)
        .fontWeight(FontWeight.Bold)
        .margin({{ top: 100 }})
    }}
    .width('100%')
    .height('100%')
    .justifyContent(FlexAlign.Start)
    .alignItems(HorizontalAlign.Center)
  }}
}}
"#,
            self.name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_generator_creates_files() {
        let gen = ArkProjectGenerator::new("MyApp");
        let files = gen.generate();

        // Root files
        assert!(files.contains_key("build-profile.json5"));
        assert!(files.contains_key("oh-package.json5"));
        assert!(files.contains_key("hvigorfile.ts"));
        assert!(files.contains_key("code-linter.json5"));

        // AppScope
        assert!(files.contains_key("AppScope/app.json5"));
        assert!(files.contains_key("AppScope/resources/base/element/string.json"));

        // Hvigor
        assert!(files.contains_key("hvigor/hvigor-config.json5"));

        // Entry module
        assert!(files.contains_key("entry/build-profile.json5"));
        assert!(files.contains_key("entry/hvigorfile.ts"));
        assert!(files.contains_key("entry/src/main/module.json5"));
        assert!(files.contains_key("entry/src/main/ets/pages/Index.ets"));
        assert!(files.contains_key("entry/src/main/ets/entryability/EntryAbility.ets"));
        assert!(files.contains_key("entry/src/main/resources/base/profile/main_pages.json"));
    }

    #[test]
    fn test_custom_package() {
        let gen = ArkProjectGenerator::with_package("MyApp", "com.company.myapp");
        assert_eq!(gen.package, "com.company.myapp");
    }

    #[test]
    fn test_index_page_has_entry_decorator() {
        let gen = ArkProjectGenerator::new("TestApp");
        let files = gen.generate();
        let index = files.get("entry/src/main/ets/pages/Index.ets").unwrap();

        assert!(index.contains("@Entry"));
        assert!(index.contains("@Component"));
        assert!(index.contains("struct Index"));
    }

    #[test]
    fn test_app_json5_has_bundle_name() {
        let gen = ArkProjectGenerator::new("TestApp");
        let files = gen.generate();
        let app_json = files.get("AppScope/app.json5").unwrap();

        assert!(app_json.contains("bundleName"));
        assert!(app_json.contains(&gen.package));
    }
}
