import { Button } from '@kit.ArkUI';

import { ChildWidget } from './ChildWidget';

@Preview
@Component
export struct ParentWidget {
    @Provide("pathStack") pathStack: NavPathStack = new NavPathStack()

  build() {
    Column() {
          ChildWidget()
        }
        .width('100%')
        .height('100%')
  }
}
import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct ChildWidget {
    @Consume("pathStack") pathStack: NavPathStack
    @State count: number = 0

  build() {
    Column() {
          Text('Child')
                .fontSize(20)
        }
        .width('100%')
        .height('100%')
        .padding(16)
  }
}
