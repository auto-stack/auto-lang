import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct TestFormWidgets {
  build() {
    NavDestination() {
      Column() {
              Checkbox()
              Toggle()
            }
    }
  }
}
