import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct TestForLoop {
    @State items: any = ["A", "B", "C"]

  build() {
    Column() {
                ForEach(this.items, (item: any) => {
                  Text()
                  /* Unknown component: item */
                }, (item: any): string => item.id)
        }
  }
}
