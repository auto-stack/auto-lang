import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct TestBox {
  build() {
    NavDestination() {
      Stack() {
              Text('Overlay')
            }
    }
  }
}
