package space.maxus.terralink.util;

import org.jetbrains.annotations.Contract;
import org.jetbrains.annotations.Nullable;
import space.maxus.terralink.TerraLink;

import java.util.concurrent.atomic.AtomicReference;
import java.util.concurrent.atomic.AtomicReferenceFieldUpdater;

public class MpscChannel<T> {
    @SuppressWarnings("AtomicFieldUpdaterNotStaticFinal")
    private final AtomicReferenceFieldUpdater<T, T> itemUpdater;
    private T head;
    private final AtomicReference<T> tail;

    public MpscChannel(AtomicReferenceFieldUpdater<T, T> updater) {
        itemUpdater = updater;
        tail = new AtomicReference<>();
    }

    public final boolean send(T item) {
        if(itemUpdater.get(item) != null) {
            TerraLink.LOGGER.warn("Tried to send object to MPSC Channel but it was held by updater!");
            return true;
        }

        for(;;) {
            final T tail = this.tail.get();
            if(this.tail.compareAndSet(tail, item)) {
                if(tail == null) {
                    this.head = item;
                    return true;
                } else {
                    itemUpdater.set(tail, item);
                    return false;
                }
            }
        }
    }

    @Contract(pure = true)
    public final @Nullable T get() {
        if(this.head == null) {
            TerraLink.LOGGER.warn("Tried to get element from empty MPSC channel!");
            return null;
        }
        return this.head;
    }

    @SuppressWarnings("StatementWithEmptyBody")
    public final T getNext() {
        if(this.head == null) {
            TerraLink.LOGGER.warn("Tried to get element from empty MPSC channel!");
            return null;
        }

        final T head = this.head;
        T next = itemUpdater.get(head);
        if(next == null) {
            this.head = null;
            if(this.tail.compareAndSet(head, null))
                return null;
            while((next = itemUpdater.get(head)) == null);
        }
        itemUpdater.lazySet(head, null);
        this.head = next;
        return head;
    }
}
