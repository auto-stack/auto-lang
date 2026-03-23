import { Button } from '@kit.ArkUI';

import { TestItem } from './TestItem';

@Preview
@Component
export struct TestForLoop {
    @State items: any = null

  build() {
    Column() {
                ForEach(this.items, (item: any) => {
                  TestItem({ title: item.title, brief: item.brief })
                }}, (item: any, index: number) => item.id ?? index
        }
  }
}
import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct TestItem {
    @State title: string = ""
    @State brief: string = ""

  build() {
    Row() {
          Text(this.title)
          Text(this.brief)
        }
  }
}
