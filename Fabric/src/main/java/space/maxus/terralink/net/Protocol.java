package space.maxus.terralink.net;

import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;
import space.maxus.terralink.TerraLink;
import space.maxus.terralink.net.packets.PacketAdvance;
import space.maxus.terralink.net.packets.PacketConnect;
import space.maxus.terralink.net.packets.PacketDisconnect;

import java.io.BufferedInputStream;
import java.io.IOException;
import java.util.HashMap;

public class Protocol {
    private static final HashMap<Byte, PacketReader<Packet>> specification = new HashMap<>();

    public static void addPacket(@NotNull PacketID id, PacketReader<Packet> reader) {
        specification.put(id.byteId, reader);
    }

    public static @Nullable Packet readPacket(@NotNull PacketID id, BufferedInputStream stream) {
        try {
            return specification.get(id.byteId).readPacket(stream);
        } catch (IOException e) {
            TerraLink.LOGGER.warn("Tried to read packet " + id.name() + " but got null!");
            return null;
        }
    }

    static {
        addPacket(PacketID.Connect, PacketReader.nullReader());
        addPacket(PacketID.Disconnect, new PacketDisconnect.Reader());
        addPacket(PacketID.Advance, new PacketAdvance.Reader());
    }

    public enum PacketID {
        Connect,
        Disconnect,
        Advance

        ;

        public final byte byteId;

        PacketID() {
            byteId = (byte) ordinal();
        }
    }
}
