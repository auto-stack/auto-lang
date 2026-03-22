import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct TestImage {
  build() {
    NavDestination() {
      Image('https://example.com/logo.png')
    }
  }
}
