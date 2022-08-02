package space.maxus.terralink.net.packets;

import space.maxus.terralink.net.BufferUtil;
import space.maxus.terralink.net.Packet;
import space.maxus.terralink.net.PacketReader;

import java.io.BufferedInputStream;
import java.io.BufferedOutputStream;
import java.io.IOException;

public class PacketAdvance extends Packet {
    private final String info;

    public PacketAdvance(String info) {
        this.info = info;
    }

    public String getBridgeInfo() {
        return info;
    }

    @Override
    public PacketReader<Packet> getReader() {
        return null;
    }

    @Override
    public void writeSelf(BufferedOutputStream os) {

    }

    public static final class Reader implements PacketReader<Packet> {

        @Override
        public Packet readPacket(BufferedInputStream is) throws IOException {
            var info = BufferUtil.readStr(is);
            return new PacketAdvance(info);
        }
    }
}
