import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct TestRow {
  build() {
    Row() {
          Text('Left')
          Text('Right')
        }
  }
}
