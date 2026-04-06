import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct NavTest {
    @State items: Object = ["Apple", "Banana", "Cherry"]

  build() {
        Column() {
          ForEach(this.items, (item: Object) => {
            Text()
    /* Unknown component: item */
          }, (item: Object): string => item.id)
        }
  }
}
