package space.maxus.terralink.net;

import com.google.common.util.concurrent.ThreadFactoryBuilder;
import org.jctools.queues.MpmcArrayQueue;
import org.jctools.queues.MpscArrayQueue;
import space.maxus.terralink.TerraLink;
import space.maxus.terralink.net.packets.PacketAdvance;
import space.maxus.terralink.net.packets.PacketConnect;
import space.maxus.terralink.net.packets.PacketDisconnect;

import java.io.BufferedInputStream;
import java.io.BufferedOutputStream;
import java.io.IOException;
import java.net.Socket;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Semaphore;

public class LinkerClient {
    private BufferedOutputStream outputStream;
    private BufferedInputStream inputStream;
    private Socket socket;

    public static final ExecutorService networkExecutor = Executors.newFixedThreadPool(4, new ThreadFactoryBuilder().setNameFormat("TerraLink Network").build());

    public void sendPacket(Packet packet) {
        try {
            packet.writeSelf(outputStream);
            outputStream.flush();
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
    }

    @SuppressWarnings("StatementWithEmptyBody")
    public Packet readPacket() {
        int id;
        try {
            while((id = inputStream.read()) == 0);
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
        if(id == -1)
            return null;
        return Protocol.readPacket(Protocol.PacketID.values()[id], inputStream);
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
            outputStream = new BufferedOutputStream(socket.getOutputStream(), 1024);
            inputStream = new BufferedInputStream(socket.getInputStream(), 1024);
        } catch (IOException e) {
            TerraLink.LOGGER.error("Could not retrieve input/output streams from socket: " + e.getMessage());
            return;
        }

        TerraLink.LOGGER.info("Starting network threads...");

        networkExecutor.execute(this::doHandshake);
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
