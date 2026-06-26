<script setup lang="ts">
import { Sidebar, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupContent, SidebarGroupLabel, SidebarHeader, SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarProvider } from '@/components/ui/sidebar'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const sidebarBasicAutoCode = `sidebar-provider {
    sidebar {
        sidebar-header {
            sidebar-menu {
                sidebar-menu-item {
                    sidebar-menu-button {
                        "Dashboard"
                    }
                }
            }
        }
        sidebar-content {
            sidebar-group {
                sidebar-group-label (text: "Main") {}
                sidebar-group-content {
                    sidebar-menu {
                        sidebar-menu-item {
                            sidebar-menu-button {
                                "Home"
                            }
                        }
                        sidebar-menu-item {
                            sidebar-menu-button {
                                "Settings"
                            }
                        }
                    }
                }
            }
        }
        sidebar-footer {
            "Footer"
        }
    }
}
`
const sidebarBasicVueCode = `<SidebarProvider>
  <Sidebar>
    <SidebarHeader>
      <SidebarMenu>
        <SidebarMenuItem>
          <SidebarMenuButton>
            Dashboard
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarHeader>
    <SidebarContent>
      <SidebarGroup>
        <SidebarGroupLabel>Main</SidebarGroupLabel>
        <SidebarGroupContent>
          <SidebarMenu>
            <SidebarMenuItem>
              <SidebarMenuButton>
                Home
              </SidebarMenuButton>
            </SidebarMenuItem>
            <SidebarMenuItem>
              <SidebarMenuButton>
                Settings
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>
    </SidebarContent>
    <SidebarFooter>
      Footer
    </SidebarFooter>
  </Sidebar>
</SidebarProvider>
`
</script>

<template>
  <ComponentDocPage title="Sidebar" description="A composable sidebar component with support for different sections and navigation items." installCommand="npx shadcn-vue@latest add sidebar">
    <DemoSection title="Simple" id="sidebar-basic" :autoCode="sidebarBasicAutoCode" :vueCode="sidebarBasicVueCode">
      <template #preview>
        <SidebarProvider>
              <Sidebar>
                <SidebarHeader>
                  <SidebarMenu>
                    <SidebarMenuItem>
                      <SidebarMenuButton>
                        Dashboard
                      </SidebarMenuButton>
                    </SidebarMenuItem>
                  </SidebarMenu>
                </SidebarHeader>
                <SidebarContent>
                  <SidebarGroup>
                    <SidebarGroupLabel>Main</SidebarGroupLabel>
                    <SidebarGroupContent>
                      <SidebarMenu>
                        <SidebarMenuItem>
                          <SidebarMenuButton>
                            Home
                          </SidebarMenuButton>
                        </SidebarMenuItem>
                        <SidebarMenuItem>
                          <SidebarMenuButton>
                            Settings
                          </SidebarMenuButton>
                        </SidebarMenuItem>
                      </SidebarMenu>
                    </SidebarGroupContent>
                  </SidebarGroup>
                </SidebarContent>
                <SidebarFooter>
                  Footer
                </SidebarFooter>
              </Sidebar>
            </SidebarProvider>
      </template>
    </DemoSection>
    <template #properties>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Property</TableHead>
            <TableHead>Type</TableHead>
            <TableHead>Default</TableHead>
            <TableHead>Values</TableHead>
            <TableHead>Description</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow>
            <TableCell>side</TableCell>
            <TableCell>string</TableCell>
            <TableCell>"left"</TableCell>
            <TableCell>"left", "right"</TableCell>
            <TableCell>Which side the sidebar appears</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>variant</TableCell>
            <TableCell>string</TableCell>
            <TableCell>"sidebar"</TableCell>
            <TableCell>"sidebar", "floating", "inset"</TableCell>
            <TableCell>Sidebar style variant</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>collapsible</TableCell>
            <TableCell>string</TableCell>
            <TableCell>"offcanvas"</TableCell>
            <TableCell>"offcanvas", "icon", "none"</TableCell>
            <TableCell>Collapsible behavior</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </template>
  </ComponentDocPage>
</template>

<style scoped>
/* Override Prism.js default styles */
pre[class*="language-"] {
  margin: 0;
}

/* Component styles */

</style>
