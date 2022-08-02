package space.maxus.terralink.net;

import com.google.common.util.concurrent.ThreadFactoryBuilder;
import space.maxus.terralink.TerraLink;
import space.maxus.terralink.net.packets.PacketAdvance;
import space.maxus.terralink.net.packets.PacketConnect;
import space.maxus.terralink.net.packets.PacketDisconnect;
import space.maxus.terralink.util.MpscChannel;

import java.io.BufferedInputStream;
import java.io.BufferedOutputStream;
import java.io.IOException;
import java.net.Socket;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Semaphore;
import java.util.concurrent.atomic.AtomicReferenceFieldUpdater;

public class LinkerClient {
    private BufferedOutputStream outputStream;
    private BufferedInputStream inputStream;
    private Socket socket;

    private final MpscChannel<Packet> packetsTx = new MpscChannel<>(AtomicReferenceFieldUpdater.newUpdater(Packet.class, Packet.class, "nextPacket"));
    private final MpscChannel<Packet> packetsRx = new MpscChannel<>(AtomicReferenceFieldUpdater.newUpdater(Packet.class, Packet.class, "nextPacket"));
    private final Semaphore txLock = new Semaphore(16);
    private final Semaphore rxLock = new Semaphore(16);
    public static final ExecutorService networkExecutor = Executors.newFixedThreadPool(4, new ThreadFactoryBuilder().setNameFormat("TerraLink Network").build());

    public void sendPacket(Packet packet) {
        if(packetsTx.send(packet))
            txLock.release();
    }

    public Packet readPacket() {
        try {
            rxLock.acquire();
            return packetsRx.get();
        } catch (InterruptedException e) {
            TerraLink.LOGGER.warn("Could not acquire RX Semaphore lock: " + e.getMessage());
            return null;
        }
    }

    public void connect() {
        try {
            socket = new Socket("127.0.0.1", 25535);
        } catch (Exception e) {
            TerraLink.LOGGER.error("Could not connect to bridge: " + e.getMessage());
            return;
        }
        TerraLink.LOGGER.info("Connected to bridge at 127.0.0.1:25535");

        try {
            outputStream = new BufferedOutputStream(socket.getOutputStream());
            inputStream = new BufferedInputStream(socket.getInputStream());
        } catch (IOException e) {
            TerraLink.LOGGER.error("Could not retrieve input/output streams from socket: " + e.getMessage());
            return;
        }

        TerraLink.LOGGER.info("Starting network threads...");

        networkExecutor.execute(this::reader);
        networkExecutor.execute(this::writer);
        networkExecutor.execute(this::doHandshake);
    }

    private void reader() {
        while(socket.isConnected()) {
            try {
                var id = inputStream.read();
                var packet = Protocol.readPacket(Protocol.PacketID.values()[id], inputStream);
                if(packetsRx.send(packet))
                    rxLock.release();
            } catch (IOException e) {
                TerraLink.LOGGER.warn(e.getMessage());
            }
        }
    }

    private void writer() {
        while(socket.isConnected()) {
            try {
                txLock.acquire();
                var next = packetsTx.get();
                if(next == null)
                    continue;
                do {
                    try {
                        next.writeSelf(outputStream);
                    } catch (IOException e) {
                        TerraLink.LOGGER.warn("Could not write packet " + next + ": " + e.getMessage());
                    }
                    next = packetsTx.getNext();
                } while(next != null);
            } catch (InterruptedException err) {
                TerraLink.LOGGER.warn("Could not acquire TX Semaphore lock!");
            }
        }
    }

    private void doHandshake() {
        this.sendPacket(new PacketConnect());

        var read = this.readPacket();
        if(read instanceof PacketDisconnect disconnect) {
            TerraLink.LOGGER.warn("Bridge disconnected client: " + disconnect.getReason());
            try {
                socket.close();
            } catch (IOException ignored) { }
        } else if(read instanceof PacketAdvance advance) {
            TerraLink.LOGGER.info("Successfully completed handshake with bridge!");
            TerraLink.LOGGER.info("Bridge: " + advance.getBridgeInfo());
        }
    }
}
