import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct TestImage {
  build() {
    Image('https://example.com/logo.png')
  }
}
