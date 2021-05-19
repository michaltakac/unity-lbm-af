using System;
using System.Runtime.InteropServices;
using System.Text;
using UnityEngine;

public class LBM
{
    private static IntPtr _sim_handle;
	private static IntPtr _data_handle;
	private UInt32 sim_timestep = 0;

	[DllImport("lbmaf")]
    private static extern bool init_sim(out IntPtr sim_handle, out IntPtr data_handle,
            UInt32 width, UInt32 height, Float32 inflow_ux,
            Float32 omega, UInt32 obstacle_x, UInt32 obstacle_y, UInt32 obstacle_r);
	public static bool InitSimulation(UInt32 w, UInt32 h, Float32 rho0, Float32 in_ux,
            Float32 om, UInt32 obs_x, UInt32 obs_y, UInt32 obs_r)
    {
        return init_sim(out _sim_handle, out _data_handle, w, h, rho0, in_ux,
            om, obs_x, obs_y, obs_r);
    }

	[DllImport("lbmaf")]
    private static extern void dispose(out IntPtr sim_handle);
	public static void DisposeSim()
    {
        dispose(out _sim_handle);
    }

	[DllImport("lbmaf")]
    private static extern void simulate(IntPtr handle, Float32 inflow_density, Float32 inflow_ux, Float32 omega);
	public static void SimulateNextIteration(Float32 in_rho, Float32 in_ux, Float32 om)
    {
        simulate(_sim_handle, in_rho, in_ux, om);
    }

	[DllImport("lbmaf")]
    private static extern void get_sim_data(IntPtr handle);
	public static void GetSimData()
    {
        get_sim_data(_sim_handle);
    }

    public void CopyResultsToBuffer(byte[] buffer, UInt32 size) {
        Marshal.Copy((IntPtr)_data_handle, buffer, 0, size);
    }
}