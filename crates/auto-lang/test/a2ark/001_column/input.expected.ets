import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct TestColumn {
  build() {
    NavDestination() {
      Column() {
              Text('Hello')
              Text('World')
            }
    }
  }
}
