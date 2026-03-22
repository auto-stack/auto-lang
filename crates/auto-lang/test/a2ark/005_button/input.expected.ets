import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct TestButton {
  build() {
    NavDestination() {
      Button('Submit')
    }
  }
}
