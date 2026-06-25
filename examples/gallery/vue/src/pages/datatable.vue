<script setup lang="ts">
import { ref } from 'vue'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { DropdownMenu } from '@/components/ui/dropdown-menu'
import { Input } from '@/components/ui/input'
import { Select } from '@/components/ui/select'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const sortColumn = ref<string>('')
const sortDirection = ref<string>('asc')
const filterText = ref<string>('')
const selectedRows = ref<number>(0)
const currentPage = ref<number>(1)
function onToggleSortStatus() {}
function onToggleSortEmail() {}
function onToggleSortAmount() {}

const datatableBasicAutoCode = `table {
    table-header {
        table-row {
            table-head (text: "Status") {}
            table-head (text: "Email") {}
            table-head (class: "text-right", text: "Amount") {}
        }
    }
    table-body {
        table-row {
            table-cell {
                badge (variant: "default", text: "Success") {}
            }
            table-cell (class: "lowercase", text: "ken99@yahoo.com") {}
            table-cell (text: "$316.00", class: "text-right font-medium") {}
        }
        table-row {
            table-cell {
                badge (variant: "secondary", text: "Pending") {}
            }
            table-cell (text: "abe45@gmail.com", class: "lowercase") {}
            table-cell (class: "text-right font-medium", text: "$242.00") {}
        }
        table-row {
            table-cell {
                badge (variant: "outline", text: "Processing") {}
            }
            table-cell (text: "monserrat44@gmail.com", class: "lowercase") {}
            table-cell (class: "text-right font-medium", text: "$837.00") {}
        }
    }
}
`
const datatableBasicVueCode = `<Table>
  <TableHeader>
    <TableRow>
      <TableHead>Status</TableHead>
      <TableHead>Email</TableHead>
      <TableHead class="text-right">Amount</TableHead>
    </TableRow>
  </TableHeader>
  <TableBody>
    <TableRow>
      <TableCell>
        <Badge>Success</Badge>
      </TableCell>
      <TableCell class="lowercase">ken99@yahoo.com</TableCell>
      <TableCell class="text-right font-medium">$316.00</TableCell>
    </TableRow>
    <TableRow>
      <TableCell>
        <Badge>Pending</Badge>
      </TableCell>
      <TableCell class="lowercase">abe45@gmail.com</TableCell>
      <TableCell class="text-right font-medium">$242.00</TableCell>
    </TableRow>
    <TableRow>
      <TableCell>
        <Badge>Processing</Badge>
      </TableCell>
      <TableCell class="lowercase">monserrat44@gmail.com</TableCell>
      <TableCell class="text-right font-medium">$837.00</TableCell>
    </TableRow>
  </TableBody>
</Table>
`
const datatableSortingAutoCode = `table {
    table-header {
        table-row {
            table-head {
                button (variant: "ghost", class: "-ml-4", onclick: ..ToggleSortStatus) {
                    "Status"
                    if .sortColumn == "status" {
                        if .sortDirection == "asc" {
                            icon (class: "ml-2 h-4 w-4", name: "arrow-up") {}
                        }
                        if .sortDirection == "desc" {
                            icon (class: "ml-2 h-4 w-4", name: "arrow-down") {}
                        }
                    }
                    if .sortColumn != "status" {
                        icon (class: "ml-2 h-4 w-4", name: "arrow-up-down") {}
                    }
                }
            }
            table-head {
                button (class: "-ml-4", variant: "ghost", onclick: ..ToggleSortEmail) {
                    "Email"
                    if .sortColumn == "email" {
                        if .sortDirection == "asc" {
                            icon (name: "arrow-up", class: "ml-2 h-4 w-4") {}
                        }
                        if .sortDirection == "desc" {
                            icon (name: "arrow-down", class: "ml-2 h-4 w-4") {}
                        }
                    }
                    if .sortColumn != "email" {
                        icon (name: "arrow-up-down", class: "ml-2 h-4 w-4") {}
                    }
                }
            }
            table-head (class: "text-right") {
                button (variant: "ghost", class: "-ml-4", onclick: ..ToggleSortAmount) {
                    "Amount"
                    if .sortColumn == "amount" {
                        if .sortDirection == "asc" {
                            icon (class: "ml-2 h-4 w-4", name: "arrow-up") {}
                        }
                        if .sortDirection == "desc" {
                            icon (name: "arrow-down", class: "ml-2 h-4 w-4") {}
                        }
                    }
                    if .sortColumn != "amount" {
                        icon (class: "ml-2 h-4 w-4", name: "arrow-up-down") {}
                    }
                }
            }
        }
    }
    table-body {
        if .sortColumn == "email" {
            if .sortDirection == "asc" {
                table-row {
                    table-cell {
                        badge (variant: "default", text: "Success") {}
                    }
                    table-cell (text: "abe45@gmail.com", class: "lowercase") {}
                    table-cell (text: "$242.00", class: "text-right font-medium") {}
                }
                table-row {
                    table-cell {
                        badge (text: "Failed", variant: "destructive") {}
                    }
                    table-cell (text: "carmella@hotmail.com", class: "lowercase") {}
                    table-cell (text: "$721.00", class: "text-right font-medium") {}
                }
                table-row {
                    table-cell {
                        badge (variant: "default", text: "Success") {}
                    }
                    table-cell (class: "lowercase", text: "ken99@yahoo.com") {}
                    table-cell (class: "text-right font-medium", text: "$316.00") {}
                }
            }
            if .sortDirection == "desc" {
                table-row {
                    table-cell {
                        badge (variant: "default", text: "Success") {}
                    }
                    table-cell (text: "ken99@yahoo.com", class: "lowercase") {}
                    table-cell (text: "$316.00", class: "text-right font-medium") {}
                }
                table-row {
                    table-cell {
                        badge (text: "Failed", variant: "destructive") {}
                    }
                    table-cell (text: "carmella@hotmail.com", class: "lowercase") {}
                    table-cell (class: "text-right font-medium", text: "$721.00") {}
                }
                table-row {
                    table-cell {
                        badge (variant: "default", text: "Success") {}
                    }
                    table-cell (class: "lowercase", text: "abe45@gmail.com") {}
                    table-cell (text: "$242.00", class: "text-right font-medium") {}
                }
            }
        }
        if .sortColumn == "amount" {
            if .sortDirection == "asc" {
                table-row {
                    table-cell {
                        badge (variant: "default", text: "Success") {}
                    }
                    table-cell (class: "lowercase", text: "abe45@gmail.com") {}
                    table-cell (text: "$242.00", class: "text-right font-medium") {}
                }
                table-row {
                    table-cell {
                        badge (text: "Success", variant: "default") {}
                    }
                    table-cell (class: "lowercase", text: "ken99@yahoo.com") {}
                    table-cell (class: "text-right font-medium", text: "$316.00") {}
                }
                table-row {
                    table-cell {
                        badge (text: "Failed", variant: "destructive") {}
                    }
                    table-cell (class: "lowercase", text: "carmella@hotmail.com") {}
                    table-cell (class: "text-right font-medium", text: "$721.00") {}
                }
            }
            if .sortDirection == "desc" {
                table-row {
                    table-cell {
                        badge (variant: "destructive", text: "Failed") {}
                    }
                    table-cell (class: "lowercase", text: "carmella@hotmail.com") {}
                    table-cell (class: "text-right font-medium", text: "$721.00") {}
                }
                table-row {
                    table-cell {
                        badge (text: "Success", variant: "default") {}
                    }
                    table-cell (text: "ken99@yahoo.com", class: "lowercase") {}
                    table-cell (text: "$316.00", class: "text-right font-medium") {}
                }
                table-row {
                    table-cell {
                        badge (text: "Success", variant: "default") {}
                    }
                    table-cell (class: "lowercase", text: "abe45@gmail.com") {}
                    table-cell (class: "text-right font-medium", text: "$242.00") {}
                }
            }
        }
        if .sortColumn == "status" {
            if .sortDirection == "asc" {
                table-row {
                    table-cell {
                        badge (text: "Failed", variant: "destructive") {}
                    }
                    table-cell (text: "carmella@hotmail.com", class: "lowercase") {}
                    table-cell (class: "text-right font-medium", text: "$721.00") {}
                }
                table-row {
                    table-cell {
                        badge (text: "Success", variant: "default") {}
                    }
                    table-cell (text: "abe45@gmail.com", class: "lowercase") {}
                    table-cell (class: "text-right font-medium", text: "$242.00") {}
                }
                table-row {
                    table-cell {
                        badge (variant: "default", text: "Success") {}
                    }
                    table-cell (text: "ken99@yahoo.com", class: "lowercase") {}
                    table-cell (text: "$316.00", class: "text-right font-medium") {}
                }
            }
            if .sortDirection == "desc" {
                table-row {
                    table-cell {
                        badge (text: "Success", variant: "default") {}
                    }
                    table-cell (text: "abe45@gmail.com", class: "lowercase") {}
                    table-cell (text: "$242.00", class: "text-right font-medium") {}
                }
                table-row {
                    table-cell {
                        badge (text: "Success", variant: "default") {}
                    }
                    table-cell (class: "lowercase", text: "ken99@yahoo.com") {}
                    table-cell (text: "$316.00", class: "text-right font-medium") {}
                }
                table-row {
                    table-cell {
                        badge (text: "Failed", variant: "destructive") {}
                    }
                    table-cell (class: "lowercase", text: "carmella@hotmail.com") {}
                    table-cell (text: "$721.00", class: "text-right font-medium") {}
                }
            }
        }
        if .sortColumn == "" {
            table-row {
                table-cell {
                    badge (variant: "default", text: "Success") {}
                }
                table-cell (class: "lowercase", text: "ken99@yahoo.com") {}
                table-cell (text: "$316.00", class: "text-right font-medium") {}
            }
            table-row {
                table-cell {
                    badge (text: "Success", variant: "default") {}
                }
                table-cell (class: "lowercase", text: "abe45@gmail.com") {}
                table-cell (text: "$242.00", class: "text-right font-medium") {}
            }
            table-row {
                table-cell {
                    badge (variant: "destructive", text: "Failed") {}
                }
                table-cell (text: "carmella@hotmail.com", class: "lowercase") {}
                table-cell (class: "text-right font-medium", text: "$721.00") {}
            }
        }
    }
}
`
const datatableSortingVueCode = `<Table>
  <TableHeader>
    <TableRow>
      <TableHead>
        <Button variant="ghost" @click="onToggleSortStatus">
          Status
          <template v-if="sortColumn == 'status'">
            <template v-if="sortDirection == 'asc'">
              <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up" />
            </template>
            <template v-if="sortDirection == 'desc'">
              <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-down" />
            </template>
          </template>
          <template v-if="sortColumn != 'status'">
            <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up-down" />
          </template>
        </Button>
      </TableHead>
      <TableHead>
        <Button variant="ghost" @click="onToggleSortEmail">
          Email
          <template v-if="sortColumn == 'email'">
            <template v-if="sortDirection == 'asc'">
              <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up" />
            </template>
            <template v-if="sortDirection == 'desc'">
              <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-down" />
            </template>
          </template>
          <template v-if="sortColumn != 'email'">
            <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up-down" />
          </template>
        </Button>
      </TableHead>
      <TableHead class="text-right">
        <Button variant="ghost" @click="onToggleSortAmount">
          Amount
          <template v-if="sortColumn == 'amount'">
            <template v-if="sortDirection == 'asc'">
              <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up" />
            </template>
            <template v-if="sortDirection == 'desc'">
              <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-down" />
            </template>
          </template>
          <template v-if="sortColumn != 'amount'">
            <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up-down" />
          </template>
        </Button>
      </TableHead>
    </TableRow>
  </TableHeader>
  <TableBody>
    <template v-if="sortColumn == 'email'">
      <template v-if="sortDirection == 'asc'">
        <TableRow>
          <TableCell>
            <Badge>Success</Badge>
          </TableCell>
          <TableCell class="lowercase">abe45@gmail.com</TableCell>
          <TableCell class="text-right font-medium">$242.00</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>
            <Badge>Failed</Badge>
          </TableCell>
          <TableCell class="lowercase">carmella@hotmail.com</TableCell>
          <TableCell class="text-right font-medium">$721.00</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>
            <Badge>Success</Badge>
          </TableCell>
          <TableCell class="lowercase">ken99@yahoo.com</TableCell>
          <TableCell class="text-right font-medium">$316.00</TableCell>
        </TableRow>
      </template>
      <template v-if="sortDirection == 'desc'">
        <TableRow>
          <TableCell>
            <Badge>Success</Badge>
          </TableCell>
          <TableCell class="lowercase">ken99@yahoo.com</TableCell>
          <TableCell class="text-right font-medium">$316.00</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>
            <Badge>Failed</Badge>
          </TableCell>
          <TableCell class="lowercase">carmella@hotmail.com</TableCell>
          <TableCell class="text-right font-medium">$721.00</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>
            <Badge>Success</Badge>
          </TableCell>
          <TableCell class="lowercase">abe45@gmail.com</TableCell>
          <TableCell class="text-right font-medium">$242.00</TableCell>
        </TableRow>
      </template>
    </template>
    <template v-if="sortColumn == 'amount'">
      <template v-if="sortDirection == 'asc'">
        <TableRow>
          <TableCell>
            <Badge>Success</Badge>
          </TableCell>
          <TableCell class="lowercase">abe45@gmail.com</TableCell>
          <TableCell class="text-right font-medium">$242.00</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>
            <Badge>Success</Badge>
          </TableCell>
          <TableCell class="lowercase">ken99@yahoo.com</TableCell>
          <TableCell class="text-right font-medium">$316.00</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>
            <Badge>Failed</Badge>
          </TableCell>
          <TableCell class="lowercase">carmella@hotmail.com</TableCell>
          <TableCell class="text-right font-medium">$721.00</TableCell>
        </TableRow>
      </template>
      <template v-if="sortDirection == 'desc'">
        <TableRow>
          <TableCell>
            <Badge>Failed</Badge>
          </TableCell>
          <TableCell class="lowercase">carmella@hotmail.com</TableCell>
          <TableCell class="text-right font-medium">$721.00</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>
            <Badge>Success</Badge>
          </TableCell>
          <TableCell class="lowercase">ken99@yahoo.com</TableCell>
          <TableCell class="text-right font-medium">$316.00</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>
            <Badge>Success</Badge>
          </TableCell>
          <TableCell class="lowercase">abe45@gmail.com</TableCell>
          <TableCell class="text-right font-medium">$242.00</TableCell>
        </TableRow>
      </template>
    </template>
    <template v-if="sortColumn == 'status'">
      <template v-if="sortDirection == 'asc'">
        <TableRow>
          <TableCell>
            <Badge>Failed</Badge>
          </TableCell>
          <TableCell class="lowercase">carmella@hotmail.com</TableCell>
          <TableCell class="text-right font-medium">$721.00</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>
            <Badge>Success</Badge>
          </TableCell>
          <TableCell class="lowercase">abe45@gmail.com</TableCell>
          <TableCell class="text-right font-medium">$242.00</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>
            <Badge>Success</Badge>
          </TableCell>
          <TableCell class="lowercase">ken99@yahoo.com</TableCell>
          <TableCell class="text-right font-medium">$316.00</TableCell>
        </TableRow>
      </template>
      <template v-if="sortDirection == 'desc'">
        <TableRow>
          <TableCell>
            <Badge>Success</Badge>
          </TableCell>
          <TableCell class="lowercase">abe45@gmail.com</TableCell>
          <TableCell class="text-right font-medium">$242.00</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>
            <Badge>Success</Badge>
          </TableCell>
          <TableCell class="lowercase">ken99@yahoo.com</TableCell>
          <TableCell class="text-right font-medium">$316.00</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>
            <Badge>Failed</Badge>
          </TableCell>
          <TableCell class="lowercase">carmella@hotmail.com</TableCell>
          <TableCell class="text-right font-medium">$721.00</TableCell>
        </TableRow>
      </template>
    </template>
    <template v-if="sortColumn == ''">
      <TableRow>
        <TableCell>
          <Badge>Success</Badge>
        </TableCell>
        <TableCell class="lowercase">ken99@yahoo.com</TableCell>
        <TableCell class="text-right font-medium">$316.00</TableCell>
      </TableRow>
      <TableRow>
        <TableCell>
          <Badge>Success</Badge>
        </TableCell>
        <TableCell class="lowercase">abe45@gmail.com</TableCell>
        <TableCell class="text-right font-medium">$242.00</TableCell>
      </TableRow>
      <TableRow>
        <TableCell>
          <Badge>Failed</Badge>
        </TableCell>
        <TableCell class="lowercase">carmella@hotmail.com</TableCell>
        <TableCell class="text-right font-medium">$721.00</TableCell>
      </TableRow>
    </template>
  </TableBody>
</Table>
`
const datatableFilteringAutoCode = `col (gap: "4") {
    row (class: "flex items-center gap-2") {
        text (class: "text-sm font-medium", text: "Filter:") {}
        input (class: "w-[250px]", placeholder: "Search by email...") {}
    }
    table {
        table-header {
            table-row {
                table-head (text: "Status") {}
                table-head (text: "Email") {}
                table-head (text: "Amount", class: "text-right") {}
            }
        }
        table-body {
            table-row {
                table-cell {
                    badge (text: "Success", variant: "default") {}
                }
                table-cell (text: "ken99@yahoo.com", class: "lowercase") {}
                table-cell (text: "$316.00", class: "text-right font-medium") {}
            }
            table-row {
                table-cell {
                    badge (variant: "default", text: "Success") {}
                }
                table-cell (class: "lowercase", text: "abe45@gmail.com") {}
                table-cell (text: "$242.00", class: "text-right font-medium") {}
            }
        }
    }
}
`
const datatableFilteringVueCode = `<div class="flex flex-col gap-4">
  <div class="flex flex-row gap-4 flex items-center gap-2">
    <span class="text-muted-foreground leading-7 text-sm font-medium">Filter:</span>
    <Input placeholder="Search by email..." />
  </div>
  <Table>
    <TableHeader>
      <TableRow>
        <TableHead>Status</TableHead>
        <TableHead>Email</TableHead>
        <TableHead class="text-right">Amount</TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      <TableRow>
        <TableCell>
          <Badge>Success</Badge>
        </TableCell>
        <TableCell class="lowercase">ken99@yahoo.com</TableCell>
        <TableCell class="text-right font-medium">$316.00</TableCell>
      </TableRow>
      <TableRow>
        <TableCell>
          <Badge>Success</Badge>
        </TableCell>
        <TableCell class="lowercase">abe45@gmail.com</TableCell>
        <TableCell class="text-right font-medium">$242.00</TableCell>
      </TableRow>
    </TableBody>
  </Table>
</div>
`
const datatablePaginationAutoCode = `col (gap: "4") {
    table {
        table-header {
            table-row {
                table-head (text: "Invoice") {}
                table-head (text: "Status") {}
                table-head (text: "Amount", class: "text-right") {}
            }
        }
        table-body {
            table-row {
                table-cell (class: "font-medium", text: "INV001") {}
                table-cell (text: "Paid") {}
                table-cell (class: "text-right", text: "$250.00") {}
            }
            table-row {
                table-cell (text: "INV002", class: "font-medium") {}
                table-cell (text: "Pending") {}
                table-cell (class: "text-right", text: "$150.00") {}
            }
        }
    }
    row (class: "flex items-center justify-between") {
        row (class: "flex items-center gap-2") {
            text (class: "text-sm text-muted-foreground", text: "Rows per page") {}
            select (class: "w-[70px]") {
                select-trigger {
                    select-value (text: "10") {}
                }
                select-content {
                    select-item (text: "10", value: "10") {}
                    select-item (text: "20", value: "20") {}
                }
            }
        }
        text (text: "Page 1 of 10", class: "text-sm text-muted-foreground") {}
        row (gap: "2") {
            button (size: "icon", disabled: true, variant: "outline") {
                icon (name: "chevron-left", class: "h-4 w-4") {}
            }
            button (variant: "outline", size: "icon") {
                icon (name: "chevron-right", class: "h-4 w-4") {}
            }
        }
    }
}
`
const datatablePaginationVueCode = `<div class="flex flex-col gap-4">
  <Table>
    <TableHeader>
      <TableRow>
        <TableHead>Invoice</TableHead>
        <TableHead>Status</TableHead>
        <TableHead class="text-right">Amount</TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      <TableRow>
        <TableCell class="font-medium">INV001</TableCell>
        <TableCell>Paid</TableCell>
        <TableCell class="text-right">$250.00</TableCell>
      </TableRow>
      <TableRow>
        <TableCell class="font-medium">INV002</TableCell>
        <TableCell>Pending</TableCell>
        <TableCell class="text-right">$150.00</TableCell>
      </TableRow>
    </TableBody>
  </Table>
  <div class="flex flex-row gap-4 flex items-center justify-between">
    <div class="flex flex-row gap-4 flex items-center gap-2">
      <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Rows per page</span>
      <Select>
        <div>
          <div>10</div>
        </div>
        <div>
          <div value="10">10</div>
          <div value="20">20</div>
        </div>
      </Select>
    </div>
    <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Page 1 of 10</span>
    <div class="flex flex-row gap-2">
      <Button variant="outline" size="icon" disabled>
        <span class="w-5 h-5 h-4 w-4" name="chevron-left" />
      </Button>
      <Button variant="outline" size="icon">
        <span class="w-5 h-5 h-4 w-4" name="chevron-right" />
      </Button>
    </div>
  </div>
</div>
`
const datatableSelectionAutoCode = `col (gap: "4") {
    table {
        table-header {
            table-row {
                table-head (class: "w-[50px]") {
                    checkbox {}
                }
                table-head (text: "Status") {}
                table-head (text: "Email") {}
                table-head (text: "Amount", class: "text-right") {}
            }
        }
        table-body {
            table-row {
                table-cell (class: "w-[50px]") {
                    checkbox {}
                }
                table-cell {
                    badge (text: "Success", variant: "default") {}
                }
                table-cell (text: "ken99@yahoo.com") {}
                table-cell (text: "$316.00", class: "text-right font-medium") {}
            }
            table-row {
                table-cell (class: "w-[50px]") {
                    checkbox (checked: true) {}
                }
                table-cell {
                    badge (variant: "default", text: "Success") {}
                }
                table-cell (text: "abe45@gmail.com") {}
                table-cell (text: "$242.00", class: "text-right font-medium") {}
            }
            table-row {
                table-cell (class: "w-[50px]") {
                    checkbox {}
                }
                table-cell {
                    badge (variant: "outline", text: "Processing") {}
                }
                table-cell (text: "monserrat44@gmail.com") {}
                table-cell (text: "$837.00", class: "text-right font-medium") {}
            }
        }
    }
    row (class: "flex items-center justify-between") {
        text (class: "text-sm text-muted-foreground", text: "1 of 3 row(s) selected.") {}
        row (gap: "2") {
            button (text: "Previous", size: "sm", variant: "outline") {}
            button (variant: "outline", text: "Next", size: "sm") {}
        }
    }
}
`
const datatableSelectionVueCode = `<div class="flex flex-col gap-4">
  <Table>
    <TableHeader>
      <TableRow>
        <TableHead class="w-[50px]">
          <Checkbox />
        </TableHead>
        <TableHead>Status</TableHead>
        <TableHead>Email</TableHead>
        <TableHead class="text-right">Amount</TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      <TableRow>
        <TableCell class="w-[50px]">
          <Checkbox />
        </TableCell>
        <TableCell>
          <Badge>Success</Badge>
        </TableCell>
        <TableCell>ken99@yahoo.com</TableCell>
        <TableCell class="text-right font-medium">$316.00</TableCell>
      </TableRow>
      <TableRow>
        <TableCell class="w-[50px]">
          <Checkbox :default-checked="true" />
        </TableCell>
        <TableCell>
          <Badge>Success</Badge>
        </TableCell>
        <TableCell>abe45@gmail.com</TableCell>
        <TableCell class="text-right font-medium">$242.00</TableCell>
      </TableRow>
      <TableRow>
        <TableCell class="w-[50px]">
          <Checkbox />
        </TableCell>
        <TableCell>
          <Badge>Processing</Badge>
        </TableCell>
        <TableCell>monserrat44@gmail.com</TableCell>
        <TableCell class="text-right font-medium">$837.00</TableCell>
      </TableRow>
    </TableBody>
  </Table>
  <div class="flex flex-row gap-4 flex items-center justify-between">
    <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">1 of 3 row(s) selected.</span>
    <div class="flex flex-row gap-2">
      <Button variant="outline" size="sm">Previous</Button>
      <Button variant="outline" size="sm">Next</Button>
    </div>
  </div>
</div>
`
const datatableActionsAutoCode = `table {
    table-header {
        table-row {
            table-head (text: "Invoice") {}
            table-head (text: "Status") {}
            table-head (text: "Amount", class: "text-right") {}
            table-head (text: "", class: "w-[50px]") {}
        }
    }
    table-body {
        table-row {
            table-cell (text: "INV001", class: "font-medium") {}
            table-cell {
                badge (text: "Paid", variant: "default") {}
            }
            table-cell (text: "$250.00", class: "text-right") {}
            table-cell (class: "w-[50px]") {
                dropdown-menu {
                    dropdown-menu-trigger {
                        button (variant: "ghost", size: "icon") {
                            icon (name: "more-horizontal", class: "h-4 w-4") {}
                        }
                    }
                    dropdown-menu-content {
                        dropdown-menu-item (text: "Copy ID") {}
                        dropdown-menu-item (text: "View details") {}
                        dropdown-menu-separator {}
                        dropdown-menu-item (class: "text-destructive", text: "Delete") {}
                    }
                }
            }
        }
        table-row {
            table-cell (class: "font-medium", text: "INV002") {}
            table-cell {
                badge (text: "Pending", variant: "secondary") {}
            }
            table-cell (text: "$150.00", class: "text-right") {}
            table-cell (class: "w-[50px]") {
                dropdown-menu {
                    dropdown-menu-trigger {
                        button (variant: "ghost", size: "icon") {
                            icon (class: "h-4 w-4", name: "more-horizontal") {}
                        }
                    }
                    dropdown-menu-content {
                        dropdown-menu-item (text: "Copy ID") {}
                        dropdown-menu-item (text: "View details") {}
                    }
                }
            }
        }
    }
}
`
const datatableActionsVueCode = `<Table>
  <TableHeader>
    <TableRow>
      <TableHead>Invoice</TableHead>
      <TableHead>Status</TableHead>
      <TableHead class="text-right">Amount</TableHead>
      <TableHead class="w-[50px]"></TableHead>
    </TableRow>
  </TableHeader>
  <TableBody>
    <TableRow>
      <TableCell class="font-medium">INV001</TableCell>
      <TableCell>
        <Badge>Paid</Badge>
      </TableCell>
      <TableCell class="text-right">$250.00</TableCell>
      <TableCell class="w-[50px]">
        <DropdownMenu>
          <div>
            <Button variant="ghost" size="icon">
              <span class="w-5 h-5 h-4 w-4" name="more-horizontal" />
            </Button>
          </div>
          <div>
            <div>Copy ID</div>
            <div>View details</div>
            <div />
            <div class="text-destructive">Delete</div>
          </div>
        </DropdownMenu>
      </TableCell>
    </TableRow>
    <TableRow>
      <TableCell class="font-medium">INV002</TableCell>
      <TableCell>
        <Badge>Pending</Badge>
      </TableCell>
      <TableCell class="text-right">$150.00</TableCell>
      <TableCell class="w-[50px]">
        <DropdownMenu>
          <div>
            <Button variant="ghost" size="icon">
              <span class="w-5 h-5 h-4 w-4" name="more-horizontal" />
            </Button>
          </div>
          <div>
            <div>Copy ID</div>
            <div>View details</div>
          </div>
        </DropdownMenu>
      </TableCell>
    </TableRow>
  </TableBody>
</Table>
`
const datatableVisibilityAutoCode = `col (gap: "4") {
    row (class: "flex items-center justify-between") {
        input (class: "w-[200px]", placeholder: "Search...") {}
        dropdown-menu {
            dropdown-menu-trigger {
                button (variant: "outline", size: "sm") {
                    icon (class: "mr-2 h-4 w-4", name: "columns") {}
                    "Columns"
                }
            }
            dropdown-menu-content {
                dropdown-menu-item (text: "Status") {}
                dropdown-menu-item (text: "Email") {}
                dropdown-menu-item (text: "Amount") {}
            }
        }
    }
    table {
        table-header {
            table-row {
                table-head (text: "Status") {}
                table-head (text: "Email") {}
                table-head (class: "text-right", text: "Amount") {}
            }
        }
        table-body {
            table-row {
                table-cell {
                    badge (text: "Success", variant: "default") {}
                }
                table-cell (text: "ken99@yahoo.com") {}
                table-cell (class: "text-right font-medium", text: "$316.00") {}
            }
        }
    }
}
`
const datatableVisibilityVueCode = `<div class="flex flex-col gap-4">
  <div class="flex flex-row gap-4 flex items-center justify-between">
    <Input placeholder="Search..." />
    <DropdownMenu>
      <div>
        <Button variant="outline" size="sm">
          <span class="w-5 h-5 mr-2 h-4 w-4" name="columns" />
          Columns
        </Button>
      </div>
      <div>
        <div>Status</div>
        <div>Email</div>
        <div>Amount</div>
      </div>
    </DropdownMenu>
  </div>
  <Table>
    <TableHeader>
      <TableRow>
        <TableHead>Status</TableHead>
        <TableHead>Email</TableHead>
        <TableHead class="text-right">Amount</TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      <TableRow>
        <TableCell>
          <Badge>Success</Badge>
        </TableCell>
        <TableCell>ken99@yahoo.com</TableCell>
        <TableCell class="text-right font-medium">$316.00</TableCell>
      </TableRow>
    </TableBody>
  </Table>
</div>
`
const datatableCompleteAutoCode = `col (gap: "4") {
    row (class: "flex items-center justify-between") {
        input (class: "w-[200px]", placeholder: "Filter emails...") {}
        row (gap: "2") {
            dropdown-menu {
                dropdown-menu-trigger {
                    button (size: "sm", variant: "outline") {
                        icon (class: "mr-2 h-4 w-4", name: "columns") {}
                        "View"
                    }
                }
                dropdown-menu-content (align: "end") {
                    dropdown-menu-item (text: "Status") {}
                    dropdown-menu-item (text: "Email") {}
                    dropdown-menu-item (text: "Amount") {}
                }
            }
            button (variant: "outline", size: "sm") {
                icon (class: "mr-2 h-4 w-4", name: "plus") {}
                "Add Payment"
            }
        }
    }
    table {
        table-header {
            table-row {
                table-head (class: "w-[50px]") {
                    checkbox {}
                }
                table-head {
                    button (class: "-ml-4", variant: "ghost") {
                        "Status"
                        icon (name: "arrow-up-down", class: "ml-2 h-4 w-4") {}
                    }
                }
                table-head {
                    button (variant: "ghost", class: "-ml-4") {
                        "Email"
                        icon (class: "ml-2 h-4 w-4", name: "arrow-up-down") {}
                    }
                }
                table-head (text: "Amount", class: "text-right") {}
                table-head (text: "", class: "w-[50px]") {}
            }
        }
        table-body {
            table-row {
                table-cell (class: "w-[50px]") {
                    checkbox {}
                }
                table-cell {
                    badge (text: "Success", variant: "default") {}
                }
                table-cell (text: "ken99@yahoo.com", class: "lowercase") {}
                table-cell (class: "text-right font-medium", text: "$316.00") {}
                table-cell (class: "w-[50px]") {
                    button (size: "icon", variant: "ghost") {
                        "..."
                    }
                }
            }
            table-row {
                table-cell (class: "w-[50px]") {
                    checkbox (checked: true) {}
                }
                table-cell {
                    badge (text: "Success", variant: "default") {}
                }
                table-cell (class: "lowercase", text: "abe45@gmail.com") {}
                table-cell (text: "$242.00", class: "text-right font-medium") {}
                table-cell (class: "w-[50px]") {
                    button (size: "icon", variant: "ghost") {
                        "..."
                    }
                }
            }
            table-row {
                table-cell (class: "w-[50px]") {
                    checkbox {}
                }
                table-cell {
                    badge (text: "Processing", variant: "outline") {}
                }
                table-cell (class: "lowercase", text: "monserrat44@gmail.com") {}
                table-cell (class: "text-right font-medium", text: "$837.00") {}
                table-cell (class: "w-[50px]") {
                    button (size: "icon", variant: "ghost") {
                        "..."
                    }
                }
            }
            table-row {
                table-cell (class: "w-[50px]") {
                    checkbox {}
                }
                table-cell {
                    badge (text: "Failed", variant: "destructive") {}
                }
                table-cell (class: "lowercase", text: "carmella@hotmail.com") {}
                table-cell (class: "text-right font-medium", text: "$721.00") {}
                table-cell (class: "w-[50px]") {
                    button (variant: "ghost", size: "icon") {
                        "..."
                    }
                }
            }
        }
    }
    row (class: "flex items-center justify-between") {
        text (text: "1 of 4 row(s) selected.", class: "text-sm text-muted-foreground") {}
        row (class: "flex items-center gap-6") {
            row (class: "flex items-center gap-2") {
                text (class: "text-sm text-muted-foreground", text: "Rows per page") {}
                select (class: "w-[70px]") {
                    select-trigger {
                        select-value {}
                    }
                    select-content {
                        select-item (value: "5", text: "5") {}
                        select-item (value: "10", text: "10") {}
                    }
                }
            }
            text (class: "text-sm text-muted-foreground", text: "Page 1 of 1") {}
            row (gap: "2") {
                button (variant: "outline", size: "icon", disabled: true) {
                    icon (name: "chevron-left", class: "h-4 w-4") {}
                }
                button (variant: "outline", size: "icon", disabled: true) {
                    icon (name: "chevron-right", class: "h-4 w-4") {}
                }
            }
        }
    }
}
`
const datatableCompleteVueCode = `<div class="flex flex-col gap-4">
  <div class="flex flex-row gap-4 flex items-center justify-between">
    <Input placeholder="Filter emails..." />
    <div class="flex flex-row gap-2">
      <DropdownMenu>
        <div>
          <Button variant="outline" size="sm">
            <span class="w-5 h-5 mr-2 h-4 w-4" name="columns" />
            View
          </Button>
        </div>
        <div align="end">
          <div>Status</div>
          <div>Email</div>
          <div>Amount</div>
        </div>
      </DropdownMenu>
      <Button variant="outline" size="sm">
        <span class="w-5 h-5 mr-2 h-4 w-4" name="plus" />
        Add Payment
      </Button>
    </div>
  </div>
  <Table>
    <TableHeader>
      <TableRow>
        <TableHead class="w-[50px]">
          <Checkbox />
        </TableHead>
        <TableHead>
          <Button variant="ghost">
            Status
            <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up-down" />
          </Button>
        </TableHead>
        <TableHead>
          <Button variant="ghost">
            Email
            <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up-down" />
          </Button>
        </TableHead>
        <TableHead class="text-right">Amount</TableHead>
        <TableHead class="w-[50px]"></TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      <TableRow>
        <TableCell class="w-[50px]">
          <Checkbox />
        </TableCell>
        <TableCell>
          <Badge>Success</Badge>
        </TableCell>
        <TableCell class="lowercase">ken99@yahoo.com</TableCell>
        <TableCell class="text-right font-medium">$316.00</TableCell>
        <TableCell class="w-[50px]">
          <Button variant="ghost" size="icon">
            ...
          </Button>
        </TableCell>
      </TableRow>
      <TableRow>
        <TableCell class="w-[50px]">
          <Checkbox :default-checked="true" />
        </TableCell>
        <TableCell>
          <Badge>Success</Badge>
        </TableCell>
        <TableCell class="lowercase">abe45@gmail.com</TableCell>
        <TableCell class="text-right font-medium">$242.00</TableCell>
        <TableCell class="w-[50px]">
          <Button variant="ghost" size="icon">
            ...
          </Button>
        </TableCell>
      </TableRow>
      <TableRow>
        <TableCell class="w-[50px]">
          <Checkbox />
        </TableCell>
        <TableCell>
          <Badge>Processing</Badge>
        </TableCell>
        <TableCell class="lowercase">monserrat44@gmail.com</TableCell>
        <TableCell class="text-right font-medium">$837.00</TableCell>
        <TableCell class="w-[50px]">
          <Button variant="ghost" size="icon">
            ...
          </Button>
        </TableCell>
      </TableRow>
      <TableRow>
        <TableCell class="w-[50px]">
          <Checkbox />
        </TableCell>
        <TableCell>
          <Badge>Failed</Badge>
        </TableCell>
        <TableCell class="lowercase">carmella@hotmail.com</TableCell>
        <TableCell class="text-right font-medium">$721.00</TableCell>
        <TableCell class="w-[50px]">
          <Button variant="ghost" size="icon">
            ...
          </Button>
        </TableCell>
      </TableRow>
    </TableBody>
  </Table>
  <div class="flex flex-row gap-4 flex items-center justify-between">
    <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">1 of 4 row(s) selected.</span>
    <div class="flex flex-row gap-4 flex items-center gap-6">
      <div class="flex flex-row gap-4 flex items-center gap-2">
        <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Rows per page</span>
        <Select>
          <div>
            <div />
          </div>
          <div>
            <div value="5">5</div>
            <div value="10">10</div>
          </div>
        </Select>
      </div>
      <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Page 1 of 1</span>
      <div class="flex flex-row gap-2">
        <Button variant="outline" size="icon" disabled>
          <span class="w-5 h-5 h-4 w-4" name="chevron-left" />
        </Button>
        <Button variant="outline" size="icon" disabled>
          <span class="w-5 h-5 h-4 w-4" name="chevron-right" />
        </Button>
      </div>
    </div>
  </div>
</div>
`
const codeblock2Code = `npm install @tanstack/vue-table`
</script>

<template>
  <ComponentDocPage title="Data Table" description="Powerful table and datagrids built using TanStack Table." installCommand="npx shadcn-vue@latest add table">
    <DemoSection title="Basic Table" id="datatable-basic" :autoCode="datatableBasicAutoCode" :vueCode="datatableBasicVueCode">
      <template #preview>
        <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Status</TableHead>
                  <TableHead>Email</TableHead>
                  <TableHead class="text-right">Amount</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow>
                  <TableCell>
                    <Badge>Success</Badge>
                  </TableCell>
                  <TableCell class="lowercase">ken99@yahoo.com</TableCell>
                  <TableCell class="text-right font-medium">$316.00</TableCell>
                </TableRow>
                <TableRow>
                  <TableCell>
                    <Badge>Pending</Badge>
                  </TableCell>
                  <TableCell class="lowercase">abe45@gmail.com</TableCell>
                  <TableCell class="text-right font-medium">$242.00</TableCell>
                </TableRow>
                <TableRow>
                  <TableCell>
                    <Badge>Processing</Badge>
                  </TableCell>
                  <TableCell class="lowercase">monserrat44@gmail.com</TableCell>
                  <TableCell class="text-right font-medium">$837.00</TableCell>
                </TableRow>
              </TableBody>
            </Table>
      </template>
    </DemoSection>
    <DemoSection title="Sorting" id="datatable-sorting" :autoCode="datatableSortingAutoCode" :vueCode="datatableSortingVueCode">
      <template #preview>
        <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>
                    <Button variant="ghost" @click="onToggleSortStatus">
                      Status
                      <template v-if="sortColumn == 'status'">
                        <template v-if="sortDirection == 'asc'">
                          <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up" />
                        </template>
                        <template v-if="sortDirection == 'desc'">
                          <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-down" />
                        </template>
                      </template>
                      <template v-if="sortColumn != 'status'">
                        <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up-down" />
                      </template>
                    </Button>
                  </TableHead>
                  <TableHead>
                    <Button variant="ghost" @click="onToggleSortEmail">
                      Email
                      <template v-if="sortColumn == 'email'">
                        <template v-if="sortDirection == 'asc'">
                          <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up" />
                        </template>
                        <template v-if="sortDirection == 'desc'">
                          <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-down" />
                        </template>
                      </template>
                      <template v-if="sortColumn != 'email'">
                        <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up-down" />
                      </template>
                    </Button>
                  </TableHead>
                  <TableHead class="text-right">
                    <Button variant="ghost" @click="onToggleSortAmount">
                      Amount
                      <template v-if="sortColumn == 'amount'">
                        <template v-if="sortDirection == 'asc'">
                          <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up" />
                        </template>
                        <template v-if="sortDirection == 'desc'">
                          <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-down" />
                        </template>
                      </template>
                      <template v-if="sortColumn != 'amount'">
                        <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up-down" />
                      </template>
                    </Button>
                  </TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <template v-if="sortColumn == 'email'">
                  <template v-if="sortDirection == 'asc'">
                    <TableRow>
                      <TableCell>
                        <Badge>Success</Badge>
                      </TableCell>
                      <TableCell class="lowercase">abe45@gmail.com</TableCell>
                      <TableCell class="text-right font-medium">$242.00</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell>
                        <Badge>Failed</Badge>
                      </TableCell>
                      <TableCell class="lowercase">carmella@hotmail.com</TableCell>
                      <TableCell class="text-right font-medium">$721.00</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell>
                        <Badge>Success</Badge>
                      </TableCell>
                      <TableCell class="lowercase">ken99@yahoo.com</TableCell>
                      <TableCell class="text-right font-medium">$316.00</TableCell>
                    </TableRow>
                  </template>
                  <template v-if="sortDirection == 'desc'">
                    <TableRow>
                      <TableCell>
                        <Badge>Success</Badge>
                      </TableCell>
                      <TableCell class="lowercase">ken99@yahoo.com</TableCell>
                      <TableCell class="text-right font-medium">$316.00</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell>
                        <Badge>Failed</Badge>
                      </TableCell>
                      <TableCell class="lowercase">carmella@hotmail.com</TableCell>
                      <TableCell class="text-right font-medium">$721.00</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell>
                        <Badge>Success</Badge>
                      </TableCell>
                      <TableCell class="lowercase">abe45@gmail.com</TableCell>
                      <TableCell class="text-right font-medium">$242.00</TableCell>
                    </TableRow>
                  </template>
                </template>
                <template v-if="sortColumn == 'amount'">
                  <template v-if="sortDirection == 'asc'">
                    <TableRow>
                      <TableCell>
                        <Badge>Success</Badge>
                      </TableCell>
                      <TableCell class="lowercase">abe45@gmail.com</TableCell>
                      <TableCell class="text-right font-medium">$242.00</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell>
                        <Badge>Success</Badge>
                      </TableCell>
                      <TableCell class="lowercase">ken99@yahoo.com</TableCell>
                      <TableCell class="text-right font-medium">$316.00</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell>
                        <Badge>Failed</Badge>
                      </TableCell>
                      <TableCell class="lowercase">carmella@hotmail.com</TableCell>
                      <TableCell class="text-right font-medium">$721.00</TableCell>
                    </TableRow>
                  </template>
                  <template v-if="sortDirection == 'desc'">
                    <TableRow>
                      <TableCell>
                        <Badge>Failed</Badge>
                      </TableCell>
                      <TableCell class="lowercase">carmella@hotmail.com</TableCell>
                      <TableCell class="text-right font-medium">$721.00</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell>
                        <Badge>Success</Badge>
                      </TableCell>
                      <TableCell class="lowercase">ken99@yahoo.com</TableCell>
                      <TableCell class="text-right font-medium">$316.00</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell>
                        <Badge>Success</Badge>
                      </TableCell>
                      <TableCell class="lowercase">abe45@gmail.com</TableCell>
                      <TableCell class="text-right font-medium">$242.00</TableCell>
                    </TableRow>
                  </template>
                </template>
                <template v-if="sortColumn == 'status'">
                  <template v-if="sortDirection == 'asc'">
                    <TableRow>
                      <TableCell>
                        <Badge>Failed</Badge>
                      </TableCell>
                      <TableCell class="lowercase">carmella@hotmail.com</TableCell>
                      <TableCell class="text-right font-medium">$721.00</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell>
                        <Badge>Success</Badge>
                      </TableCell>
                      <TableCell class="lowercase">abe45@gmail.com</TableCell>
                      <TableCell class="text-right font-medium">$242.00</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell>
                        <Badge>Success</Badge>
                      </TableCell>
                      <TableCell class="lowercase">ken99@yahoo.com</TableCell>
                      <TableCell class="text-right font-medium">$316.00</TableCell>
                    </TableRow>
                  </template>
                  <template v-if="sortDirection == 'desc'">
                    <TableRow>
                      <TableCell>
                        <Badge>Success</Badge>
                      </TableCell>
                      <TableCell class="lowercase">abe45@gmail.com</TableCell>
                      <TableCell class="text-right font-medium">$242.00</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell>
                        <Badge>Success</Badge>
                      </TableCell>
                      <TableCell class="lowercase">ken99@yahoo.com</TableCell>
                      <TableCell class="text-right font-medium">$316.00</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell>
                        <Badge>Failed</Badge>
                      </TableCell>
                      <TableCell class="lowercase">carmella@hotmail.com</TableCell>
                      <TableCell class="text-right font-medium">$721.00</TableCell>
                    </TableRow>
                  </template>
                </template>
                <template v-if="sortColumn == ''">
                  <TableRow>
                    <TableCell>
                      <Badge>Success</Badge>
                    </TableCell>
                    <TableCell class="lowercase">ken99@yahoo.com</TableCell>
                    <TableCell class="text-right font-medium">$316.00</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell>
                      <Badge>Success</Badge>
                    </TableCell>
                    <TableCell class="lowercase">abe45@gmail.com</TableCell>
                    <TableCell class="text-right font-medium">$242.00</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell>
                      <Badge>Failed</Badge>
                    </TableCell>
                    <TableCell class="lowercase">carmella@hotmail.com</TableCell>
                    <TableCell class="text-right font-medium">$721.00</TableCell>
                  </TableRow>
                </template>
              </TableBody>
            </Table>
      </template>
    </DemoSection>
    <DemoSection title="Filtering" id="datatable-filtering" :autoCode="datatableFilteringAutoCode" :vueCode="datatableFilteringVueCode">
      <template #preview>
        <div class="flex flex-col gap-4">
              <div class="flex flex-row gap-4 flex items-center gap-2">
                <span class="text-muted-foreground leading-7 text-sm font-medium">Filter:</span>
                <Input placeholder="Search by email..." />
              </div>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Status</TableHead>
                    <TableHead>Email</TableHead>
                    <TableHead class="text-right">Amount</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow>
                    <TableCell>
                      <Badge>Success</Badge>
                    </TableCell>
                    <TableCell class="lowercase">ken99@yahoo.com</TableCell>
                    <TableCell class="text-right font-medium">$316.00</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell>
                      <Badge>Success</Badge>
                    </TableCell>
                    <TableCell class="lowercase">abe45@gmail.com</TableCell>
                    <TableCell class="text-right font-medium">$242.00</TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </div>
      </template>
    </DemoSection>
    <DemoSection title="Pagination" id="datatable-pagination" :autoCode="datatablePaginationAutoCode" :vueCode="datatablePaginationVueCode">
      <template #preview>
        <div class="flex flex-col gap-4">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Invoice</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead class="text-right">Amount</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow>
                    <TableCell class="font-medium">INV001</TableCell>
                    <TableCell>Paid</TableCell>
                    <TableCell class="text-right">$250.00</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell class="font-medium">INV002</TableCell>
                    <TableCell>Pending</TableCell>
                    <TableCell class="text-right">$150.00</TableCell>
                  </TableRow>
                </TableBody>
              </Table>
              <div class="flex flex-row gap-4 flex items-center justify-between">
                <div class="flex flex-row gap-4 flex items-center gap-2">
                  <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Rows per page</span>
                  <Select>
                    <div>
                      <div>10</div>
                    </div>
                    <div>
                      <div value="10">10</div>
                      <div value="20">20</div>
                    </div>
                  </Select>
                </div>
                <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Page 1 of 10</span>
                <div class="flex flex-row gap-2">
                  <Button variant="outline" size="icon" disabled>
                    <span class="w-5 h-5 h-4 w-4" name="chevron-left" />
                  </Button>
                  <Button variant="outline" size="icon">
                    <span class="w-5 h-5 h-4 w-4" name="chevron-right" />
                  </Button>
                </div>
              </div>
            </div>
      </template>
    </DemoSection>
    <DemoSection title="Row Selection" id="datatable-selection" :autoCode="datatableSelectionAutoCode" :vueCode="datatableSelectionVueCode">
      <template #preview>
        <div class="flex flex-col gap-4">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead class="w-[50px]">
                      <Checkbox />
                    </TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead>Email</TableHead>
                    <TableHead class="text-right">Amount</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow>
                    <TableCell class="w-[50px]">
                      <Checkbox />
                    </TableCell>
                    <TableCell>
                      <Badge>Success</Badge>
                    </TableCell>
                    <TableCell>ken99@yahoo.com</TableCell>
                    <TableCell class="text-right font-medium">$316.00</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell class="w-[50px]">
                      <Checkbox :default-checked="true" />
                    </TableCell>
                    <TableCell>
                      <Badge>Success</Badge>
                    </TableCell>
                    <TableCell>abe45@gmail.com</TableCell>
                    <TableCell class="text-right font-medium">$242.00</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell class="w-[50px]">
                      <Checkbox />
                    </TableCell>
                    <TableCell>
                      <Badge>Processing</Badge>
                    </TableCell>
                    <TableCell>monserrat44@gmail.com</TableCell>
                    <TableCell class="text-right font-medium">$837.00</TableCell>
                  </TableRow>
                </TableBody>
              </Table>
              <div class="flex flex-row gap-4 flex items-center justify-between">
                <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">1 of 3 row(s) selected.</span>
                <div class="flex flex-row gap-2">
                  <Button variant="outline" size="sm">Previous</Button>
                  <Button variant="outline" size="sm">Next</Button>
                </div>
              </div>
            </div>
      </template>
    </DemoSection>
    <DemoSection title="Row Actions" id="datatable-actions" :autoCode="datatableActionsAutoCode" :vueCode="datatableActionsVueCode">
      <template #preview>
        <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Invoice</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead class="text-right">Amount</TableHead>
                  <TableHead class="w-[50px]"></TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow>
                  <TableCell class="font-medium">INV001</TableCell>
                  <TableCell>
                    <Badge>Paid</Badge>
                  </TableCell>
                  <TableCell class="text-right">$250.00</TableCell>
                  <TableCell class="w-[50px]">
                    <DropdownMenu>
                      <div>
                        <Button variant="ghost" size="icon">
                          <span class="w-5 h-5 h-4 w-4" name="more-horizontal" />
                        </Button>
                      </div>
                      <div>
                        <div>Copy ID</div>
                        <div>View details</div>
                        <div />
                        <div class="text-destructive">Delete</div>
                      </div>
                    </DropdownMenu>
                  </TableCell>
                </TableRow>
                <TableRow>
                  <TableCell class="font-medium">INV002</TableCell>
                  <TableCell>
                    <Badge>Pending</Badge>
                  </TableCell>
                  <TableCell class="text-right">$150.00</TableCell>
                  <TableCell class="w-[50px]">
                    <DropdownMenu>
                      <div>
                        <Button variant="ghost" size="icon">
                          <span class="w-5 h-5 h-4 w-4" name="more-horizontal" />
                        </Button>
                      </div>
                      <div>
                        <div>Copy ID</div>
                        <div>View details</div>
                      </div>
                    </DropdownMenu>
                  </TableCell>
                </TableRow>
              </TableBody>
            </Table>
      </template>
    </DemoSection>
    <DemoSection title="Column Visibility" id="datatable-visibility" :autoCode="datatableVisibilityAutoCode" :vueCode="datatableVisibilityVueCode">
      <template #preview>
        <div class="flex flex-col gap-4">
              <div class="flex flex-row gap-4 flex items-center justify-between">
                <Input placeholder="Search..." />
                <DropdownMenu>
                  <div>
                    <Button variant="outline" size="sm">
                      <span class="w-5 h-5 mr-2 h-4 w-4" name="columns" />
                      Columns
                    </Button>
                  </div>
                  <div>
                    <div>Status</div>
                    <div>Email</div>
                    <div>Amount</div>
                  </div>
                </DropdownMenu>
              </div>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Status</TableHead>
                    <TableHead>Email</TableHead>
                    <TableHead class="text-right">Amount</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow>
                    <TableCell>
                      <Badge>Success</Badge>
                    </TableCell>
                    <TableCell>ken99@yahoo.com</TableCell>
                    <TableCell class="text-right font-medium">$316.00</TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </div>
      </template>
    </DemoSection>
    <DemoSection title="Complete Example" id="datatable-complete" :autoCode="datatableCompleteAutoCode" :vueCode="datatableCompleteVueCode">
      <template #preview>
        <div class="flex flex-col gap-4">
              <div class="flex flex-row gap-4 flex items-center justify-between">
                <Input placeholder="Filter emails..." />
                <div class="flex flex-row gap-2">
                  <DropdownMenu>
                    <div>
                      <Button variant="outline" size="sm">
                        <span class="w-5 h-5 mr-2 h-4 w-4" name="columns" />
                        View
                      </Button>
                    </div>
                    <div align="end">
                      <div>Status</div>
                      <div>Email</div>
                      <div>Amount</div>
                    </div>
                  </DropdownMenu>
                  <Button variant="outline" size="sm">
                    <span class="w-5 h-5 mr-2 h-4 w-4" name="plus" />
                    Add Payment
                  </Button>
                </div>
              </div>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead class="w-[50px]">
                      <Checkbox />
                    </TableHead>
                    <TableHead>
                      <Button variant="ghost">
                        Status
                        <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up-down" />
                      </Button>
                    </TableHead>
                    <TableHead>
                      <Button variant="ghost">
                        Email
                        <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up-down" />
                      </Button>
                    </TableHead>
                    <TableHead class="text-right">Amount</TableHead>
                    <TableHead class="w-[50px]"></TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow>
                    <TableCell class="w-[50px]">
                      <Checkbox />
                    </TableCell>
                    <TableCell>
                      <Badge>Success</Badge>
                    </TableCell>
                    <TableCell class="lowercase">ken99@yahoo.com</TableCell>
                    <TableCell class="text-right font-medium">$316.00</TableCell>
                    <TableCell class="w-[50px]">
                      <Button variant="ghost" size="icon">
                        ...
                      </Button>
                    </TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell class="w-[50px]">
                      <Checkbox :default-checked="true" />
                    </TableCell>
                    <TableCell>
                      <Badge>Success</Badge>
                    </TableCell>
                    <TableCell class="lowercase">abe45@gmail.com</TableCell>
                    <TableCell class="text-right font-medium">$242.00</TableCell>
                    <TableCell class="w-[50px]">
                      <Button variant="ghost" size="icon">
                        ...
                      </Button>
                    </TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell class="w-[50px]">
                      <Checkbox />
                    </TableCell>
                    <TableCell>
                      <Badge>Processing</Badge>
                    </TableCell>
                    <TableCell class="lowercase">monserrat44@gmail.com</TableCell>
                    <TableCell class="text-right font-medium">$837.00</TableCell>
                    <TableCell class="w-[50px]">
                      <Button variant="ghost" size="icon">
                        ...
                      </Button>
                    </TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell class="w-[50px]">
                      <Checkbox />
                    </TableCell>
                    <TableCell>
                      <Badge>Failed</Badge>
                    </TableCell>
                    <TableCell class="lowercase">carmella@hotmail.com</TableCell>
                    <TableCell class="text-right font-medium">$721.00</TableCell>
                    <TableCell class="w-[50px]">
                      <Button variant="ghost" size="icon">
                        ...
                      </Button>
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
              <div class="flex flex-row gap-4 flex items-center justify-between">
                <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">1 of 4 row(s) selected.</span>
                <div class="flex flex-row gap-4 flex items-center gap-6">
                  <div class="flex flex-row gap-4 flex items-center gap-2">
                    <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Rows per page</span>
                    <Select>
                      <div>
                        <div />
                      </div>
                      <div>
                        <div value="5">5</div>
                        <div value="10">10</div>
                      </div>
                    </Select>
                  </div>
                  <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Page 1 of 1</span>
                  <div class="flex flex-row gap-2">
                    <Button variant="outline" size="icon" disabled>
                      <span class="w-5 h-5 h-4 w-4" name="chevron-left" />
                    </Button>
                    <Button variant="outline" size="icon" disabled>
                      <span class="w-5 h-5 h-4 w-4" name="chevron-right" />
                    </Button>
                  </div>
                </div>
              </div>
            </div>
      </template>
    </DemoSection>

  </ComponentDocPage>
</template>

<style scoped>
/* Override Prism.js default styles */
pre[class*="language-"] {
  margin: 0;
}

/* Component styles */

</style>

