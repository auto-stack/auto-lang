import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct TestRow {
  build() {
    NavDestination() {
      Row() {
              Text('Left')
              Text('Right')
            }
    }
  }
}
