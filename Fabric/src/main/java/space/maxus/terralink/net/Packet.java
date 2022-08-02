package space.maxus.terralink.net;

import java.io.BufferedOutputStream;
import java.io.IOException;

public abstract class Packet {
    public volatile Packet nextPacket;

    public abstract PacketReader<Packet> getReader();
    public abstract void writeSelf(BufferedOutputStream os) throws IOException;
}
