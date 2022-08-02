package space.maxus.terralink.net.packets;

import space.maxus.terralink.net.BufferUtil;
import space.maxus.terralink.net.Packet;
import space.maxus.terralink.net.PacketReader;
import space.maxus.terralink.net.Protocol;

import java.io.BufferedInputStream;
import java.io.BufferedOutputStream;
import java.io.IOException;

public class PacketDisconnect extends Packet {
    private final String reason;

    public String getReason() {
        return reason;
    }

    public PacketDisconnect(String reason) {
        this.reason = reason;
    }

    @Override
    public PacketReader<Packet> getReader() {
        return new Reader();
    }

    @Override
    public void writeSelf(BufferedOutputStream os) throws IOException {
        os.write(Protocol.PacketID.Disconnect.byteId);
        BufferUtil.writeStr(os, reason);
    }

    public static final class Reader implements PacketReader<Packet> {
        @Override
        public Packet readPacket(BufferedInputStream is) throws IOException {
            return new PacketDisconnect(BufferUtil.readStr(is));
        }
    }
}
