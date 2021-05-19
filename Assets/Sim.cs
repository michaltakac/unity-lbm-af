using System;
using System.Collections;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using UnityEngine;

public class Sim : MonoBehaviour
{
    public UInt32 domain_width = 128;
    public UInt32 domain_height = 128;

    // Physical parameters.
    public Float32 initial_density = 1.0f;

    // x location of the cylinder
    public UInt32 obstacle_x = domain_width / 5 + 1;
    // y location of the cylinder
    public UInt32 obstacle_y = domain_height / 2 + domain_height / 30;
    // radius of the cylinder
    public UInt32 obstacle_r = domain_height / 10 + 1;


	// Lattice speed
    public Float32 inflow_density = 1.0f;
	public Float32 inflow_speed = 0.1f;
    // Reynolds number
    public Float32 re = 220.0f;
    // Kinematic viscosity
    public Float32 nu = inflow_speed * 2.0f * obstacle_r / re;
    // Relaxation time
    public Float32 tau = 3.0f * nu + 0.5f;
    // Relaxation parameter
    public Float32 omega = 1.0f / tau;

    private static byte[] buffer;
    private UInt32 size;
    private Texture2D image;


    void Start()
    {
        size = domain_width * domain_height * 4;
        buffer = new byte[size];
        image = new Texture2D(domain_width, domain_height, TextureFormat.RGBA32, false);
        GetComponent<Renderer>().material.mainTexture = image;

        bool isSimReady = InitSimulation(
            domain_width, domain_height, initial_density, inflow_speed,
            omega, obstacle_x, obstacle_y, obstacle_r);
        if (!isSimReady) return;

        Debug.Log("Simulation initialized.");
        StartCoroutine("GetData");
    }


    public IEnumerator GetData()
    {
        while (true)
        {
            LBM.SimulateNextIteration();
            LBM.GetSimData();
            LBM.CopyResultsToBuffer((IntPtr)_data_handle, buffer, size);
            image.LoadRawTextureData(buffer);
            image.Apply();
            sim_timestep++;
            yield return null;
        }
    }

    void Update()
    {

    }

    void OnDestroy()
    {
        // DisposeSim(); // TODO: implement disposing on Rust side
    }
}
