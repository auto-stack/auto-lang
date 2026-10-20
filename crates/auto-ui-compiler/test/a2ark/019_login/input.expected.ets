import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct TestGridRow {
  build() {
        GridRow() {
          GridCol() {
            Column() {
              Text('Hello')
            }
            .width('100%')
          }
        }
  }
}
