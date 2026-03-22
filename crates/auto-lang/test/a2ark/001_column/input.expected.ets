import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct TestColumn {
  build() {
    Column() {
          Text('Hello')
          Text('World')
        }
  }
}
