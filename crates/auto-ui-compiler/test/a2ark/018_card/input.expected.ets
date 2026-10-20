import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct CardTest {
  build() {
        Column() {
          Text('Card Title')
          Text('Card content goes here')
        }
        .padding(16)
        .borderRadius(12)
        .backgroundColor(Color.White)
        .shadow({ radius: 12, color: '#00000020', offsetX: 2, offsetY: 4 })
  }
}
