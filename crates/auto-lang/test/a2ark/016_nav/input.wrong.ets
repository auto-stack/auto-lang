import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct NavTest {
    @State items: any = ["Apple", "Banana", "Cherry"]

  build() {
    Column() {
                ForEach(this.items, (item: any) => {
                  Text()
                  /* Unknown component: item */
                }, (item: any): string => item.id)
        }
  }
}
