import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct TestTabs {
    @State activeTab: string = 'tab1'

  @State currentIndex: number = 0
  private tabsController: TabsController = new TabsController()

  build() {
    Column() {
          Tabs() {
                  Row() {
                            Text()
                            /* Unknown component: tab1 */
                            Text()
                            /* Unknown component: tab2 */
                          }
                  TabContent()
                  /* Unknown component: tab1 */
                  TabContent()
                  /* Unknown component: tab2 */
                }
        }
  }
}
