<!-- Products01Page component -->
<script setup lang="ts">
import { ref, computed } from 'vue'
import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle } from '@/components/ui/alert-dialog'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Table } from '@/components/ui/table'

interface Product {
  id: string
  name: string
  status: string
  price: string
  stock: number
  category: string
}

const products = ref<Product[]>([
  { id: '1', name: 'Premium Headphones', status: 'In Stock', price: '$299.00', stock: 143, category: 'Electronics' },
  { id: '2', name: 'Mechanical Keyboard', status: 'In Stock', price: '$149.00', stock: 89, category: 'Electronics' },
  { id: '3', name: 'Wireless Mouse', status: 'Low Stock', price: '$79.00', stock: 12, category: 'Electronics' },
  { id: '4', name: 'USB-C Hub', status: 'In Stock', price: '$59.00', stock: 234, category: 'Accessories' },
  { id: '5', name: 'Monitor Stand', status: 'Out of Stock', price: '$89.00', stock: 0, category: 'Furniture' },
  { id: '6', name: 'Desk Lamp LED', status: 'In Stock', price: '$45.00', stock: 67, category: 'Furniture' },
])

const search = ref<string>('')

const filteredProducts = computed(() => {
  if (!search.value) return products.value
  const q = search.value.toLowerCase()
  return products.value.filter(p =>
    p.name.toLowerCase().includes(q) || p.category.toLowerCase().includes(q) || p.status.toLowerCase().includes(q)
  )
})

// --- Delete ---
const deleteTarget = ref<Product | null>(null)
const deleteOpen = ref(false)

function confirmDelete(product: Product) {
  deleteTarget.value = product
  deleteOpen.value = true
}

function doDelete() {
  if (!deleteTarget.value) return
  products.value = products.value.filter(p => p.id !== deleteTarget.value!.id)
  deleteTarget.value = null
  deleteOpen.value = false
}

// --- Edit ---
const editOpen = ref(false)
const editForm = ref<Omit<Product, 'id'> & { id: string }>({ id: '', name: '', status: 'In Stock', price: '', stock: 0, category: '' })
const editError = ref('')

function openEdit(product: Product) {
  editForm.value = { ...product }
  editError.value = ''
  editOpen.value = true
}

function saveEdit() {
  const f = editForm.value
  if (!f.name.trim()) { editError.value = 'Product name is required'; return }
  if (!f.price.trim()) { editError.value = 'Price is required'; return }
  editError.value = ''
  const idx = products.value.findIndex(p => p.id === f.id)
  if (idx !== -1) {
    products.value[idx] = { ...f, stock: Number(f.stock) || 0 }
  }
  editOpen.value = false
}

// --- Add ---
const addOpen = ref(false)
const addForm = ref({ name: '', status: 'In Stock', price: '', stock: 0, category: '' })
const addError = ref('')

function openAdd() {
  addForm.value = { name: '', status: 'In Stock', price: '', stock: 0, category: '' }
  addError.value = ''
  addOpen.value = true
}

function saveAdd() {
  const f = addForm.value
  if (!f.name.trim()) { addError.value = 'Product name is required'; return }
  if (!f.price.trim()) { addError.value = 'Price is required'; return }
  addError.value = ''
  products.value.push({
    id: String(Date.now()),
    name: f.name.trim(),
    status: f.status,
    price: f.price.trim(),
    stock: Number(f.stock) || 0,
    category: f.category.trim(),
  })
  addOpen.value = false
}
</script>

<template>
 <div class="flex flex-col pb-8">
 <h1 class="text-4xl font-bold tracking-tight">Products</h1>
 <span class="text-muted-foreground">Manage your product inventory.</span>
 <div class="flex flex-col w-full p-6 gap-6 mt-6">
 <div class="flex flex-row items-center justify-between gap-4">
 <Input v-model="search" placeholder="Search products..." class="max-w-sm" />
 <Button @click="openAdd">Add Product</Button>
 </div>
 <Card>
 <div class="flex flex-col p-0">
 <Table class="w-full">
 <thead class="bg-muted/50">
 <tr class="border-b border-border">
 <th class="border px-4 py-2 text-left font-semibold h-12 px-4 text-left text-sm font-medium text-muted-foreground"><span>Name</span></th>
 <th class="border px-4 py-2 text-left font-semibold h-12 px-4 text-left text-sm font-medium text-muted-foreground"><span>Status</span></th>
 <th class="border px-4 py-2 text-left font-semibold h-12 px-4 text-left text-sm font-medium text-muted-foreground"><span>Price</span></th>
 <th class="border px-4 py-2 text-left font-semibold h-12 px-4 text-left text-sm font-medium text-muted-foreground"><span>Stock</span></th>
 <th class="border px-4 py-2 text-left font-semibold h-12 px-4 text-left text-sm font-medium text-muted-foreground"><span>Category</span></th>
 <th class="border px-4 py-2 text-left font-semibold h-12 px-4 text-left text-sm font-medium text-muted-foreground"><span>Actions</span></th>
 </tr>
 </thead>
 <tbody>
 <tr v-if="filteredProducts.length === 0">
 <td colspan="6" class="border px-4 py-8 text-center text-muted-foreground">No products found.</td>
 </tr>
 <tr class="border-b border-border" v-for="product in filteredProducts" :key="product.id">
 <td class="border px-4 py-2 p-4 text-sm font-medium">{{ product.name }}</td>
 <td class="border px-4 py-2 p-4">
 <Badge class="bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400" v-if="product.status === 'In Stock'">{{ product.status }}</Badge>
 <Badge class="bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400" v-else-if="product.status === 'Low Stock'">{{ product.status }}</Badge>
 <Badge class="bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400" v-else>{{ product.status }}</Badge>
 </td>
 <td class="border px-4 py-2 p-4 text-sm">{{ product.price }}</td>
 <td class="border px-4 py-2 p-4 text-sm">{{ product.stock }}</td>
 <td class="border px-4 py-2 p-4 text-sm">{{ product.category }}</td>
 <td class="border px-4 py-2 p-4">
 <div class="flex flex-row gap-2">
 <Button class="h-8 px-2 text-xs" @click="openEdit(product)">Edit</Button>
 <Button class="h-8 px-2 text-xs bg-red-600 hover:bg-red-700 text-white" @click="confirmDelete(product)">Delete</Button>
 </div>
 </td>
 </tr>
 </tbody>
 </Table>
 </div>
 </Card>
 </div>

 <!-- Delete Confirmation Dialog -->
 <AlertDialog v-model:open="deleteOpen">
 <AlertDialogContent>
 <AlertDialogHeader>
 <AlertDialogTitle>Delete Product</AlertDialogTitle>
 <AlertDialogDescription>
 Are you sure you want to delete <strong>{{ deleteTarget?.name }}</strong>? This action cannot be undone.
 </AlertDialogDescription>
 </AlertDialogHeader>
 <AlertDialogFooter>
 <AlertDialogCancel>Cancel</AlertDialogCancel>
 <AlertDialogAction class="bg-red-600 hover:bg-red-700 text-white" @click="doDelete">Delete</AlertDialogAction>
 </AlertDialogFooter>
 </AlertDialogContent>
 </AlertDialog>

 <!-- Edit Dialog -->
 <Dialog v-model:open="editOpen">
 <DialogContent class="sm:max-w-[425px]">
 <DialogHeader>
 <DialogTitle>Edit Product</DialogTitle>
 <DialogDescription>Update the product details below.</DialogDescription>
 </DialogHeader>
 <div v-if="editError" class="rounded-md bg-red-50 dark:bg-red-900/20 p-3 text-sm text-red-600 dark:text-red-400">{{ editError }}</div>
 <div class="flex flex-col gap-4 py-4">
 <div class="flex flex-col gap-2">
 <Label for="edit-name">Name</Label>
 <Input id="edit-name" v-model="editForm.name" placeholder="Product name" />
 </div>
 <div class="flex flex-col gap-2">
 <Label for="edit-price">Price</Label>
 <Input id="edit-price" v-model="editForm.price" placeholder="$0.00" />
 </div>
 <div class="flex flex-col gap-2">
 <Label for="edit-stock">Stock</Label>
 <Input id="edit-stock" v-model.number="editForm.stock" type="number" placeholder="0" />
 </div>
 <div class="flex flex-col gap-2">
 <Label for="edit-category">Category</Label>
 <Input id="edit-category" v-model="editForm.category" placeholder="Category" />
 </div>
 <div class="flex flex-col gap-2">
 <Label for="edit-status">Status</Label>
 <Select v-model="editForm.status">
 <SelectTrigger id="edit-status"><SelectValue placeholder="Select status" /></SelectTrigger>
 <SelectContent>
 <SelectItem value="In Stock">In Stock</SelectItem>
 <SelectItem value="Low Stock">Low Stock</SelectItem>
 <SelectItem value="Out of Stock">Out of Stock</SelectItem>
 </SelectContent>
 </Select>
 </div>
 </div>
 <DialogFooter>
 <Button variant="outline" @click="editOpen = false">Cancel</Button>
 <Button @click="saveEdit">Save Changes</Button>
 </DialogFooter>
 </DialogContent>
 </Dialog>

 <!-- Add Dialog -->
 <Dialog v-model:open="addOpen">
 <DialogContent class="sm:max-w-[425px]">
 <DialogHeader>
 <DialogTitle>Add Product</DialogTitle>
 <DialogDescription>Fill in the details for the new product.</DialogDescription>
 </DialogHeader>
 <div v-if="addError" class="rounded-md bg-red-50 dark:bg-red-900/20 p-3 text-sm text-red-600 dark:text-red-400">{{ addError }}</div>
 <div class="flex flex-col gap-4 py-4">
 <div class="flex flex-col gap-2">
 <Label for="add-name">Name</Label>
 <Input id="add-name" v-model="addForm.name" placeholder="Product name" />
 </div>
 <div class="flex flex-col gap-2">
 <Label for="add-price">Price</Label>
 <Input id="add-price" v-model="addForm.price" placeholder="$0.00" />
 </div>
 <div class="flex flex-col gap-2">
 <Label for="add-stock">Stock</Label>
 <Input id="add-stock" v-model.number="addForm.stock" type="number" placeholder="0" />
 </div>
 <div class="flex flex-col gap-2">
 <Label for="add-category">Category</Label>
 <Input id="add-category" v-model="addForm.category" placeholder="Category" />
 </div>
 <div class="flex flex-col gap-2">
 <Label for="add-status">Status</Label>
 <Select v-model="addForm.status">
 <SelectTrigger id="add-status"><SelectValue placeholder="Select status" /></SelectTrigger>
 <SelectContent>
 <SelectItem value="In Stock">In Stock</SelectItem>
 <SelectItem value="Low Stock">Low Stock</SelectItem>
 <SelectItem value="Out of Stock">Out of Stock</SelectItem>
 </SelectContent>
 </Select>
 </div>
 </div>
 <DialogFooter>
 <Button variant="outline" @click="addOpen = false">Cancel</Button>
 <Button @click="saveAdd">Add Product</Button>
 </DialogFooter>
 </DialogContent>
 </Dialog>
 </div>

</template>

<style scoped>
/* Component styles */

</style>
