# LayerScript Language: Standard Algorithms Cookbook

This page provides advanced, production-grade algorithms written in LayerScript, demonstrating low-level concurrency, SIMD vector math, and zero-overhead array sorting.

---

## 🗺️ Algorithms List
1. [Lock-Free Concurrent Stack](#1-lock-free-concurrent-stack)
2. [Bounded Bubble Sort (Zero Bounds Checking)](#2-bounded-bubble-sort-zero-bounds-checking)
3. [SIMD Vector Float Addition](#3-simd-vector-float-addition)

---

## 1. Lock-Free Concurrent Stack
A thread-safe, lock-free Stack (Treiber Stack) using atomic Compare-And-Swap (CAS) pointers from the `core::atomic` namespace.

```layerscript
import core::atomic::{atomic_compare_exchange_u64, atomic_load_u64};

struct Node<T> {
    T value;
    Node<T>* next;
}

pub struct LockFreeStack<T> {
    Node<T>* head;
}

pub function push<T>(stack: LockFreeStack<T>*, value: T) {
    // Allocate a new Node on the heap
    var new_node: Node<T>* = core::memory::raw_alloc(Node<T>::size()) as Node<T>*;
    new_node.value = value;
    
    var mut CAS_success: b1 = false;
    while (!CAS_success) {
        // Load the current head pointer atomically
        var old_head: Node<T>* = atomic_load_u64(&(stack.head) as u64*) as Node<T>*;
        new_node.next = old_head;
        
        // Attempt to atomically swap the head pointer to our new node
        CAS_success = atomic_compare_exchange_u64(
            &(stack.head) as u64*,
            old_head as u64,
            new_node as u64
        );
    }
}

pub function pop<T>(stack: LockFreeStack<T>*, out_val: T*) -> b1 {
    var mut CAS_success: b1 = false;
    while (!CAS_success) {
        var old_head: Node<T>* = atomic_load_u64(&(stack.head) as u64*) as Node<T>*;
        if (old_head == 0 as Node<T>*) {
            return false; // Stack is empty
        }
        
        var next_node: Node<T>* = old_head.next;
        
        // Attempt to swap head to old_head.next
        CAS_success = atomic_compare_exchange_u64(
            &(stack.head) as u64*,
            old_head as u64,
            next_node as u64
        );
        
        if (CAS_success) {
            // Write the value to the out pointer and free the node memory
            *out_val = old_head.value;
            core::memory::raw_free(old_head as usize);
            return true;
        }
    }
    return false;
}
```

---

## 2. Bounded Bubble Sort (Zero Bounds Checking)
This sorting algorithm utilizes refinement types to guarantee to the compiler that all index accesses remain strictly inside the array bounds, completely eliding bounds checks at compile time.

```layerscript
// In-place bubble sort
function bubble_sort<T, N: usize>(array: T*) {
    if (N <= 1) {
        return;
    }
    
    for (var mut i: usize = 0; i < N - 1; i++) {
        // We refine 'j' to be strictly less than N - 1 - i.
        // Because of this refinement, the SMT solver proves that:
        // 1. j < N
        // 2. j + 1 < N
        // Thus, both array accesses compile without bounds-checking guards.
        for (var mut j: usize = 0; j < N - 1 - i) {
            var next_idx: usize = j + 1;
            
            if (array[j] > array[next_idx]) {
                // Swap elements
                var temp: T = array[j];
                array[j] = array[next_idx];
                array[next_idx] = temp;
            }
        }
    }
}
```

---

## 3. SIMD Vector Float Addition
Demonstrates utilizing the `core::simd` registers and load/store APIs to perform parallel vector math on aligned array buffers.

```layerscript
import core::simd::{f32x4, simd_load_aligned_f32, simd_add_f32, simd_store_aligned_f32};

// Performs element-wise vector addition: C = A + B
// A, B, and C must be aligned to 16-byte boundaries.
function vector_add_aligned(A: f32*, B: f32*, C: f32*, count: usize where count % 4 == 0) {
    for (var mut i: usize = 0; i < count; i += 4) {
        // 1. Load 4 floats from A and B into 128-bit SIMD registers
        var reg_a: f32x4 = simd_load_aligned_f32(A + i);
        var reg_b: f32x4 = simd_load_aligned_f32(B + i);
        
        // 2. Execute parallel vector add (single instruction)
        var reg_sum: f32x4 = simd_add_f32(reg_a, reg_b);
        
        // 3. Write vector register output back to memory C
        simd_store_aligned_f32(C + i, reg_sum);
    }
}
```
